use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::task::failure::Failure;

#[derive(Debug)]
pub struct Worktrees(Vec<PathBuf>);

impl Worktrees {
    const PREFIX: &str = "worktree ";

    pub fn below(&self, parent: &Path) -> Vec<PathBuf> {
        self.0.iter().filter(|path| path.starts_with(parent)).cloned().collect()
    }
}

impl FromStr for Worktrees {
    type Err = Failure;

    fn from_str(listing: &str) -> Result<Worktrees, Failure> {
        Ok(Worktrees(listing.lines().filter_map(|line| line.strip_prefix(Self::PREFIX)).map(PathBuf::from).collect()))
    }
}
