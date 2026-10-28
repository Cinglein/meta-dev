mod walker;

#[cfg(test)]
mod freeze_tests;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Mutex, PoisonError};

use meta_door::kinds::Kind;
use meta_door::{Change, Door};

use super::alternates::Alternates;
use super::cause::Cause;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::git_call::GitCall;
use super::inspection::Inspection;
use super::kept::Kept;
use super::occupancy::Occupancy;
use super::thaw::{Thaw, Thawed};
use crate::request::local::LocalFile;
use crate::request::local::local_clone::quarantine::Stamp;
use walker::Walker;

#[derive(Debug, Default)]
pub struct Host {
    lenders: Mutex<HashMap<PathBuf, Vec<Alternates>>>,
}

impl Host {
    const LISTING: [&'static str; 3] = ["-w", "-F", "n"];
    const RECURSIVE: &'static str = "+D";

    fn ran(mut command: Command, input: Option<&str>) -> std::io::Result<Output> {
        let piped = input.map_or_else(Stdio::null, |_| Stdio::piped());
        let mut child = command.stdin(piped).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        input.zip(child.stdin.take()).map(|(input, mut pipe)| pipe.write_all(input.as_bytes())).transpose()?;
        child.wait_with_output()
    }

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
        EntryKind::from(Kind::found(Walker::inspected(path)))
    }

    fn canonical(&self, path: &Path) -> Option<PathBuf> {
        let admin = || Door::inspected(path).map(|admin| admin.path().to_path_buf());
        Door::canonical(path).or_else(|_| admin()).ok()
    }

    fn text(&self, path: &Path) -> Option<String> {
        let stored = || Thaw::stored(path).map_or_else(|| Door::read(path), |ledger| ledger.and_then(Door::read));
        let read = || Door::admin_text(path).unwrap_or_else(stored);
        (self.entry(path) == EntryKind::File).then(read).and_then(Result::ok)
    }

    fn git(&self, call: &GitCall) -> Result<String, Kept> {
        let command = call.command(std::env::vars_os().map(|(name, _)| name));
        let ran = Self::ran(command, call.input());
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
        let mut known = self.lenders.lock().unwrap_or_else(PoisonError::into_inner);
        known.entry(root.to_path_buf()).or_insert_with(|| self.lenders(root)).clone()
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

    fn frozen(&self, quarantine: &Path, plan: Thaw) -> Result<Thaw, Kept> {
        let found = self.walked(quarantine).into_iter();
        let files = found.filter(|found| found.kind() == EntryKind::File && found.shared().is_none());
        let originals = files.map(|found| {
            let file = quarantine.join(found.path());
            Walker::inspected(&file).map(|entry| Thawed::new(file, &entry.permissions()))
        });
        let thaw = plan.holding(
            originals.collect::<std::io::Result<_>>().map_err(|failure| Kept::Uninspected(Cause::new(failure)))?,
        );
        let ledger = serde_json::to_string(&thaw).map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        let entry = thaw.ledger().map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        LocalFile::new(entry).replaced(&ledger).map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        let failed = thaw.files().iter().find_map(|thawed| {
            let mut read_only = thawed.permissions();
            read_only.set_readonly(true);
            Door::changed(thawed.path(), Change::Permissions(read_only)).err()
        });
        failed.map_or(Ok(thaw.clone()), |failure| {
            self.thawed(&thaw);
            Err(Kept::Uninspected(Cause::new(failure)))
        })
    }

    fn thawed(&self, thaw: &Thaw) {
        thaw.files()
            .iter()
            .for_each(|thawed| drop(Door::changed(thawed.path(), Change::Permissions(thawed.permissions()))));
        drop(thaw.ledger().and_then(|ledger| Door::changed(ledger, Change::RemoveFile)));
    }

    fn stamp(&self) -> Stamp {
        Stamp::current()
    }
}
