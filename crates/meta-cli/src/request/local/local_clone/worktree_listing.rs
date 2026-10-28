use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeListing(String);

impl WorktreeListing {
    const PREFIX: &'static str = "worktree ";

    pub fn paths(&self, checkout: &Path) -> Vec<PathBuf> {
        self.0
            .lines()
            .filter_map(|line| line.strip_prefix(Self::PREFIX))
            .map(|path| meta_door::Door::canonical(path).unwrap_or_else(|_| PathBuf::from(path)))
            .filter(|path| path != checkout)
            .collect()
    }
}

impl From<String> for WorktreeListing {
    fn from(listed: String) -> Self {
        Self(listed)
    }
}

impl AsRef<str> for WorktreeListing {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
