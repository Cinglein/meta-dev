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

    pub fn main(&self) -> Option<&Path> {
        self.0.last().map(PathBuf::as_path)
    }
}

impl FromStr for Worktrees {
    type Err = Failure;

    fn from_str(listing: &str) -> Result<Worktrees, Failure> {
        Ok(Worktrees(listing.lines().filter_map(|line| line.strip_prefix(Self::PREFIX)).map(PathBuf::from).collect()))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Worktrees;

    const LISTING: &str =
        "worktree /repo\nHEAD 0a1b\nbranch refs/heads/main\n\nworktree /tmp/gate/x\nHEAD 0a1b\ndetached\n";
    const MAIN: &str = "/repo";
    const GATES: &str = "/tmp/gate";
    const LINKED: &str = "/tmp/gate/x";

    #[test]
    fn the_main_worktree_comes_first_and_linked_ones_are_found_below_a_parent() {
        let listed: Worktrees = LISTING.parse().unwrap();
        assert_eq!(listed.main(), Some(Path::new(MAIN)));
        assert_eq!(listed.below(Path::new(GATES)), [Path::new(LINKED)]);
    }
}
