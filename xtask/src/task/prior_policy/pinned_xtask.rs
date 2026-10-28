use std::fs;
use std::path::Path;

use meta_severity::Scale;
use toml::{Table, Value};

use super::PriorPolicy;
use crate::task::revision::{Commit, Revision, TrackedPath};

#[derive(Debug, PartialEq, Eq)]
pub enum PinnedXtask {
    Unpinned,
    Git(Commit),
}

impl PinnedXtask {
    const LOCK: &str = "Cargo.lock";
    const PACKAGE: &str = "meta-xtask";
    const GIT: &str = "git+";
    const COMMIT: char = '#';

    pub fn read_at(revision: &Revision) -> PinnedXtask {
        let lock = revision.present(Path::new(Self::LOCK)).ok().flatten();
        lock.as_deref().map_or(PinnedXtask::Unpinned, Self::in_lock)
    }

    pub fn read_in(root: &Path) -> PinnedXtask {
        let lock = fs::read_to_string(root.join(Self::LOCK)).ok();
        lock.as_deref().map_or(PinnedXtask::Unpinned, Self::in_lock)
    }

    pub fn in_lock(text: &str) -> PinnedXtask {
        text.parse::<Table>()
            .ok()
            .and_then(|lock| {
                lock.get("package")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .find(|package| package.get("name").and_then(Value::as_str) == Some(Self::PACKAGE))
                    .and_then(|package| package.get("source").and_then(Value::as_str))
                    .and_then(|source| source.strip_prefix(Self::GIT))
                    .and_then(|source| source.rsplit_once(Self::COMMIT))
                    .and_then(|(_, commit)| commit.parse().ok())
            })
            .map_or(PinnedXtask::Unpinned, PinnedXtask::Git)
    }

    pub fn shipped_levels(&self, now: &PinnedXtask, databases: &Path) -> Option<Scale> {
        self.moved_from(now).and_then(|then| Self::shipped_in(then, databases))
    }

    fn moved_from(&self, now: &PinnedXtask) -> Option<&Commit> {
        match self {
            PinnedXtask::Git(then) if self != now => Some(then),
            PinnedXtask::Git(_) | PinnedXtask::Unpinned => None,
        }
    }

    fn shipped_in(commit: &Commit, databases: &Path) -> Option<Scale> {
        fs::read_dir(databases)
            .into_iter()
            .flatten()
            .flatten()
            .find_map(|entry| {
                Revision::in_database(&entry.path(), commit.clone())
                    .tracked(TrackedPath::new(Path::new(PriorPolicy::DEFAULT_LEVELS)))
                    .ok()
                    .flatten()
            })
            .and_then(|text| serde_json::from_str(&text).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::PinnedXtask;

    const EARLY: &str = "0123abc";
    const LOCKS: [&str; 4] = [
        "[[package]]\nname = \"meta-xtask\"\nsource = \"git+https://example.invalid/meta-dev?tag=v0#0123abc\"\n",
        "[[package]]\nname = \"meta-xtask\"\nsource = \"git+https://example.invalid/meta-dev?tag=v0#4567def\"\n",
        "[[package]]\nname = \"meta-xtask\"\n",
        "",
    ];

    #[test]
    fn only_a_git_pin_that_moved_since_the_merge_base_asks_for_its_shipped_levels() {
        let [early, late, unsourced, unlocked] = LOCKS.map(PinnedXtask::in_lock);
        let moved = [(&early, &late), (&early, &early), (&unsourced, &late), (&unlocked, &early), (&early, &unlocked)]
            .map(|(then, now)| then.moved_from(now).map(ToString::to_string));
        assert!(unsourced == PinnedXtask::Unpinned && unlocked == PinnedXtask::Unpinned);
        assert_eq!(moved, [Some(EARLY.to_owned()), None, None, None, Some(EARLY.to_owned())]);
    }
}
