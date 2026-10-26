mod walker;

#[cfg(test)]
mod freeze_tests;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::alternates::Alternates;
use super::cause::Cause;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::git_call::GitCall;
use super::inspection::Inspection;
use super::kept::Kept;
use super::occupancy::Occupancy;
use super::thaw::{Thaw, Thawed};
use crate::request::local::local_clone::quarantine::Stamp;
use walker::Walker;

#[derive(Debug, Default)]
pub struct Host {
    lenders: RefCell<HashMap<PathBuf, Vec<Alternates>>>,
}

impl Host {
    const LISTING: [&'static str; 3] = ["-w", "-F", "n"];
    const RECURSIVE: &'static str = "+D";

    fn lenders(&self, root: &Path) -> Vec<Alternates> {
        let found = self.walked(root).into_iter().filter(|found| found.kind() == EntryKind::File && found.lends());
        let listed = found.map(|found| root.join(found.path())).filter_map(|file| {
            let objects = file.parent().and_then(Path::parent)?.to_path_buf();
            self.text(&file).map(|listed| Alternates::new(objects, &listed))
        });
        listed.collect()
    }
}

impl Inspection for Host {
    fn entry(&self, path: &Path) -> EntryKind {
        match path.symlink_metadata() {
            Ok(entry) if entry.is_symlink() => EntryKind::Symlink,
            Ok(entry) if entry.is_dir() => EntryKind::Dir,
            Ok(_) => EntryKind::File,
            Err(failure) if failure.kind() == ErrorKind::NotFound => EntryKind::Missing,
            Err(_) => EntryKind::Unreadable,
        }
    }

    fn canonical(&self, path: &Path) -> Option<PathBuf> {
        path.canonicalize().ok()
    }

    fn text(&self, path: &Path) -> Option<String> {
        (self.entry(path) == EntryKind::File).then(|| std::fs::read_to_string(path).ok()).flatten()
    }

    fn git(&self, call: &GitCall) -> Result<String, Kept> {
        let ran = call.command(std::env::vars_os().map(|(name, _)| name)).output();
        let ran = ran.map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        let shown = |stream: &[u8]| String::from_utf8_lossy(stream).into_owned();
        ran.status
            .success()
            .then(|| shown(&ran.stdout))
            .ok_or_else(|| Kept::Uninspected(Cause::new(shown(&ran.stderr))))
    }

    fn walked(&self, path: &Path) -> Vec<Found> {
        Walker::new(path).descended(Path::new(""))
    }

    fn neighbours(&self, root: &Path) -> Vec<Alternates> {
        let known = self.lenders.borrow().get(root).cloned();
        known.unwrap_or_else(|| {
            let lenders = self.lenders(root);
            self.lenders.borrow_mut().insert(root.to_path_buf(), lenders.clone());
            lenders
        })
    }

    fn occupancy(&self, paths: &[PathBuf]) -> Occupancy {
        let scanned = paths.iter().flat_map(|path| [OsStr::new(Self::RECURSIVE), path.as_os_str()]);
        match Command::new("lsof").args(Self::LISTING).args(scanned).output() {
            Ok(listed) if listed.stderr.is_empty() => {
                let Ok(occupancy) = String::from_utf8_lossy(&listed.stdout).parse();
                occupancy
            }
            Ok(listed) => Occupancy::Unlisted(Cause::new(String::from_utf8_lossy(&listed.stderr))),
            Err(failure) => Occupancy::Unlisted(Cause::new(failure)),
        }
    }

    fn frozen(&self, path: &Path) -> Result<Thaw, Kept> {
        let files = self.walked(path).into_iter().filter(|found| found.kind() == EntryKind::File);
        let frozen: Vec<std::io::Result<Thawed>> = files
            .map(|found| path.join(found.path()))
            .map(|file| {
                let original = file.symlink_metadata()?.permissions();
                let mut read_only = original.clone();
                read_only.set_readonly(true);
                std::fs::set_permissions(&file, read_only).map(|()| Thawed::new(file, original))
            })
            .collect();
        let thaw = Thaw::new(frozen.iter().filter_map(|frozen| frozen.as_ref().ok().cloned()).collect());
        frozen.into_iter().find_map(Result::err).map_or(Ok(thaw.clone()), |failure| {
            self.thawed(&thaw);
            Err(Kept::Uninspected(Cause::new(failure)))
        })
    }

    fn thawed(&self, thaw: &Thaw) {
        thaw.files().iter().for_each(|thawed| drop(std::fs::set_permissions(thawed.path(), thawed.permissions())));
    }

    fn stamp(&self) -> Stamp {
        Stamp::current()
    }
}
