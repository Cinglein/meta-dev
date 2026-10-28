use std::fs;
use std::path::Path;

use toml::{Table, Value};

use super::PriorPolicy;
use crate::task::failure::Failure;
use crate::task::revision::{Commit, Revision, TrackedPath};

#[derive(Debug, PartialEq, Eq)]
pub struct PinnedXtask(Commit);

impl PinnedXtask {
    const PACKAGE: &str = "meta-xtask";
    const GIT: &str = "git+";
    const COMMIT: char = '#';

    pub fn in_lock(text: &str) -> Result<Option<PinnedXtask>, Failure> {
        let lock: Table = text.parse().map_err(Failure::PriorLock)?;
        lock.get("package")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|package| package.get("name").and_then(Value::as_str) == Some(Self::PACKAGE))
            .find_map(|package| package.get("source").and_then(Value::as_str))
            .and_then(|source| source.strip_prefix(Self::GIT))
            .map(|source| source.rsplit_once(Self::COMMIT).ok_or(Failure::NoPinnedLevels))
            .map(|pinned| pinned.and_then(|(_, commit)| commit.parse()).map(PinnedXtask))
            .transpose()
    }

    pub fn levels(&self, databases: &Path) -> Result<String, Failure> {
        fs::read_dir(databases)
            .into_iter()
            .flatten()
            .flatten()
            .find_map(|entry| {
                Revision::in_database(entry.path(), self.0.clone())
                    .tracked(TrackedPath::new(Path::new(PriorPolicy::DEFAULT_LEVELS)))
                    .ok()
                    .flatten()
            })
            .ok_or(Failure::NoPinnedLevels)
    }
}
