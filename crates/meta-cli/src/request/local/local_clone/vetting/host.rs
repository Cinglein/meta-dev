mod walker;

#[cfg(test)]
mod tests;

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
use walker::Walker;

#[derive(Clone, Copy, Debug)]
pub struct Host;

impl Host {
    const GIT_PREFIX: &'static str = "GIT_";
    const OBJECTS: [&'static str; 2] = [".git/objects", "objects"];
    const ALTERNATES: &'static str = "info/alternates";
    const LISTING: [&'static str; 3] = ["-w", "-F", "n"];
    const RECURSIVE: &'static str = "+D";
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
        let mut command = Command::new("git");
        let cleared = std::env::vars_os()
            .map(|(name, _)| name)
            .filter(|name| name.as_encoded_bytes().starts_with(Self::GIT_PREFIX.as_bytes()))
            .fold(&mut command, |command, name| command.env_remove(name));
        let ran = cleared.args(call.arguments()).current_dir(call.directory()).output();
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
        let siblings = root.read_dir().into_iter().flatten().flatten().map(|sibling| sibling.path());
        let objects = siblings.flat_map(|sibling| Self::OBJECTS.map(|objects| sibling.join(objects)));
        objects
            .filter_map(|objects| {
                self.text(&objects.join(Self::ALTERNATES)).map(|listed| Alternates::new(objects, &listed))
            })
            .collect()
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
}
