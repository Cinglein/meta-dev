mod walker;

#[cfg(test)]
mod freeze_tests;
#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use meta_run::{EnvPolicy, Environment, Run};

use super::alternates::Alternates;
use super::cause::Cause;
use super::change::Change;
use super::entry_kind::EntryKind;
use super::found::Found;
use super::git_call::GitCall;
use super::kept::Kept;
use super::look::Look;
use super::made::Made;
use super::occupancy::Occupancy;
use super::seen::Seen;
use super::thaw::{Thaw, Thawed};
use crate::request::local::LocalFile;
use crate::request::project::CiSettings;
use walker::Walker;

// The only place gc touches the world. The interpreter is its only caller.
#[derive(Debug)]
pub struct Host {
    settings: CiSettings,
}

impl Host {
    const LSOF: &'static str = "lsof";
    const LISTING: [&'static str; 3] = ["-w", "-F", "n"];
    const RECURSIVE: &'static str = "+D";

    pub fn new(settings: CiSettings) -> Self {
        Self { settings }
    }

    pub fn seen(&self, look: &Look) -> Seen {
        match look {
            Look::Entry(path) => Seen::Entry(self.entry(path)),
            Look::Canonical(path) => Seen::Canonical(self.canonical(path)),
            Look::Text(path) => Seen::Text(self.text(path)),
            Look::Git(call) => Seen::Git(self.git(call)),
            Look::Walk(path) => Seen::Walk(self.walked(path)),
            Look::Lenders(root) => Seen::Lenders(self.neighbours(root)),
            Look::Occupancy(paths) => Seen::Occupancy(self.occupancy(paths)),
        }
    }

    pub fn made(&self, change: &Change) -> Made {
        match change {
            Change::Git(call) | Change::Destroy(call) => Made::Git(self.git(call)),
            Change::Freeze(quarantine, plan) => Made::Frozen(self.frozen(quarantine, plan.clone())),
            Change::Thaw(thaw) => {
                self.thawed(thaw);
                Made::Thawed
            }
        }
    }

    pub fn entry(&self, path: &Path) -> EntryKind {
        match path.symlink_metadata() {
            Ok(entry) if entry.is_symlink() => EntryKind::Symlink,
            Ok(entry) if entry.is_dir() => EntryKind::Dir,
            Ok(_) => EntryKind::File,
            Err(failure) if failure.kind() == ErrorKind::NotFound => EntryKind::Missing,
            Err(_) => EntryKind::Unreadable,
        }
    }

    pub fn canonical(&self, path: &Path) -> Option<PathBuf> {
        path.canonicalize().ok()
    }

    pub fn text(&self, path: &Path) -> Option<String> {
        (self.entry(path) == EntryKind::File).then(|| std::fs::read_to_string(path).ok()).flatten()
    }

    pub fn git(&self, call: &GitCall) -> Result<String, Kept> {
        let ran = call.run(Environment::current()).ran(self.settings.budget(CiSettings::ROUTINE).deadline());
        let ran = ran.map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        let output = ran.stdout().text().into_owned();
        ran.success().then_some(output).ok_or_else(|| Kept::Uninspected(Cause::new(ran.stderr())))
    }

    pub fn walked(&self, path: &Path) -> Vec<Found> {
        Walker::new(path).descended(Path::new(""))
    }

    pub fn neighbours(&self, root: &Path) -> Vec<Alternates> {
        let found = self.walked(root).into_iter().filter(|found| found.kind() == EntryKind::File && found.lends());
        let listed = found.map(|found| root.join(found.path())).filter_map(|file| {
            let objects = file.parent().and_then(Path::parent)?.to_path_buf();
            self.text(&file).map(|listed| Alternates::new(objects, &listed))
        });
        listed.collect()
    }

    pub fn occupancy(&self, paths: &[PathBuf]) -> Occupancy {
        let scanned = paths.iter().flat_map(|path| [OsStr::new(Self::RECURSIVE), path.as_os_str()]);
        let listing = Run::new(Self::LSOF, EnvPolicy::Inherit).args(Self::LISTING).args(scanned);
        match listing.ran(self.settings.budget(CiSettings::ROUTINE).deadline()) {
            Ok(listed) if listed.stderr().first_line().is_none() => {
                let Ok(occupancy) = listed.stdout().text().parse();
                occupancy
            }
            Ok(listed) => Occupancy::Unlisted(Cause::new(listed.stderr())),
            Err(failure) => Occupancy::Unlisted(Cause::new(failure)),
        }
    }

    pub fn frozen(&self, quarantine: &Path, plan: Thaw) -> Result<Thaw, Kept> {
        let found = self.walked(quarantine).into_iter();
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
            self.thawed(&thaw);
            Err(Kept::Uninspected(Cause::new(failure)))
        })
    }

    pub fn thawed(&self, thaw: &Thaw) {
        thaw.files().iter().for_each(|thawed| drop(std::fs::set_permissions(thawed.path(), thawed.permissions())));
        drop(std::fs::remove_file(thaw.ledger()));
    }
}

#[cfg(test)]
impl Default for Host {
    fn default() -> Self {
        Self::new(CiSettings::try_from(&crate::request::project::Config::default()).unwrap())
    }
}
