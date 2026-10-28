use std::fs;
use std::path::Path;

use meta_severity::Scale;
use toml::{Table, Value};

use super::PriorPolicy;
use crate::task::failure::Failure;
use crate::task::revision::{Commit, Revision, TrackedPath};

#[derive(Debug, PartialEq, Eq)]
pub enum PinnedXtask {
    Absent,
    Unpinned,
    Git(Commit),
}

impl PinnedXtask {
    const LOCK: &str = "Cargo.lock";
    const PACKAGE: &str = "meta-xtask";
    const GIT: &str = "git+";
    const COMMIT: char = '#';

    pub fn read_at(revision: &Revision) -> Result<PinnedXtask, Failure> {
        revision.present(Path::new(Self::LOCK))?.as_deref().map_or(Ok(PinnedXtask::Absent), Self::in_lock)
    }

    pub fn read_in(root: &Path) -> Result<PinnedXtask, Failure> {
        fs::read_to_string(root.join(Self::LOCK)).ok().as_deref().map_or(Ok(PinnedXtask::Absent), Self::in_lock)
    }

    pub fn in_lock(text: &str) -> Result<PinnedXtask, Failure> {
        let lock: Table = text.parse().map_err(Failure::PriorLock)?;
        lock.get("package")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|package| package.get("name").and_then(Value::as_str) == Some(Self::PACKAGE))
            .map_or(Ok(PinnedXtask::Absent), |package| {
                package
                    .get("source")
                    .and_then(Value::as_str)
                    .and_then(|source| source.strip_prefix(Self::GIT))
                    .and_then(|source| source.rsplit_once(Self::COMMIT))
                    .map_or(Ok(PinnedXtask::Unpinned), |(_, commit)| commit.parse().map(PinnedXtask::Git))
            })
    }

    pub fn levels(&self, now: &PinnedXtask, databases: &Path) -> Result<Option<Scale>, Failure> {
        match (self, now) {
            (PinnedXtask::Absent, _) => Ok(None),
            (PinnedXtask::Unpinned, PinnedXtask::Unpinned) => Ok(Some(Scale::top_default())),
            (PinnedXtask::Unpinned, _) => Err(Failure::XtaskUnpinned),
            (PinnedXtask::Git(then), PinnedXtask::Git(now)) if then == now => Ok(Some(Scale::top_default())),
            (PinnedXtask::Git(then), _) => Self::shipped_in(then, databases),
        }
    }

    fn shipped_in(commit: &Commit, databases: &Path) -> Result<Option<Scale>, Failure> {
        let seen: Vec<Option<String>> = fs::read_dir(databases)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                Revision::in_database(entry.path(), commit.clone())
                    .tracked(TrackedPath::new(Path::new(PriorPolicy::DEFAULT_LEVELS)))
                    .ok()
            })
            .collect();
        (!seen.is_empty())
            .then(|| seen.iter().flatten().next())
            .ok_or(Failure::PinnedCommitMissing)?
            .map(|text| serde_json::from_str(text).map_err(Failure::PriorLevels))
            .transpose()
    }
}
