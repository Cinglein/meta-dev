mod walker;

#[cfg(test)]
mod freeze_tests;
#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use meta_run::{Budget, EnvPolicy, Run};

use super::alternates::Alternates;
use super::cause::Cause;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::kept::Kept;
use super::occupancy::Occupancy;
use super::thaw::{Thaw, Thawed};
use crate::request::local::LocalFile;
use walker::Walker;

#[derive(Clone, Copy, Debug)]
pub struct Disk;

impl Disk {
    const LSOF: &'static str = "lsof";
    const LISTING: [&'static str; 3] = ["-w", "-F", "n"];
    const RECURSIVE: &'static str = "+D";

    pub fn entry(path: &Path) -> EntryKind {
        match path.symlink_metadata() {
            Ok(entry) if entry.is_symlink() => EntryKind::Symlink,
            Ok(entry) if entry.is_dir() => EntryKind::Dir,
            Ok(_) => EntryKind::File,
            Err(failure) if failure.kind() == ErrorKind::NotFound => EntryKind::Missing,
            Err(_) => EntryKind::Unreadable,
        }
    }

    pub fn canonical(path: &Path) -> Option<PathBuf> {
        path.canonicalize().ok()
    }

    pub fn text(path: &Path) -> Option<String> {
        (Self::entry(path) == EntryKind::File).then(|| std::fs::read_to_string(path).ok()).flatten()
    }

    pub fn walked(path: &Path) -> Vec<Found> {
        Walker::new(path).descended(Path::new(""))
    }

    pub fn lenders(root: &Path) -> Vec<Alternates> {
        let found = Self::walked(root).into_iter().filter(|found| found.kind() == EntryKind::File && found.lends());
        let listed = found.map(|found| root.join(found.path())).filter_map(|file| {
            let objects = file.parent().and_then(Path::parent)?.to_path_buf();
            Self::text(&file).map(|listed| Alternates::new(objects, &listed))
        });
        listed.collect()
    }

    pub fn occupancy(paths: &[PathBuf], budget: Budget) -> Occupancy {
        let scanned = paths.iter().flat_map(|path| [OsStr::new(Self::RECURSIVE), path.as_os_str()]);
        let listing = Run::new(Self::LSOF, EnvPolicy::Inherit).args(Self::LISTING).args(scanned);
        match listing.ran(budget.deadline()) {
            Ok(listed) if listed.stderr().first_line().is_none() => {
                let Ok(occupancy) = listed.stdout().text().parse();
                occupancy
            }
            Ok(listed) => Occupancy::Unlisted(Cause::new(listed.stderr())),
            Err(failure) => Occupancy::Unlisted(Cause::new(failure)),
        }
    }

    pub fn frozen(quarantine: &Path, plan: Thaw) -> Result<Thaw, Kept> {
        let found = Self::walked(quarantine).into_iter();
        let files = found.filter(|found| found.kind() == EntryKind::File && found.shared().is_none());
        let originals = files.map(|found| {
            let file = quarantine.join(found.path());
            file.symlink_metadata().map(|entry| Thawed::new(file, &entry.permissions()))
        });
        let thaw = plan.holding(
            originals.collect::<std::io::Result<_>>().map_err(|failure| Kept::Uninspected(Cause::new(failure)))?,
        );
        let ledger = serde_json::to_string(&thaw).map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        LocalFile::new(thaw.ledger()).replaced(&ledger).map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        let failed = thaw.files().iter().find_map(|thawed| {
            let mut read_only = thawed.permissions();
            read_only.set_readonly(true);
            std::fs::set_permissions(thawed.path(), read_only).err()
        });
        failed.map_or(Ok(thaw.clone()), |failure| {
            Self::thawed(&thaw);
            Err(Kept::Uninspected(Cause::new(failure)))
        })
    }

    pub fn thawed(thaw: &Thaw) {
        thaw.files().iter().for_each(|thawed| drop(std::fs::set_permissions(thawed.path(), thawed.permissions())));
        drop(std::fs::remove_file(thaw.ledger()));
    }
}
