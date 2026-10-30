use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Worktrees(Vec<PathBuf>);

impl Worktrees {
    pub fn below(&self, parent: &Path) -> Vec<PathBuf> {
        self.0.iter().filter(|path| path.starts_with(parent)).cloned().collect()
    }

    pub fn main(&self) -> Option<&Path> {
        self.0.first().map(PathBuf::as_path)
    }
}

impl From<Vec<PathBuf>> for Worktrees {
    fn from(paths: Vec<PathBuf>) -> Worktrees {
        Worktrees(paths)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::Worktrees;

    const MAIN: &str = "/repo";
    const GATES: &str = "/tmp/gate";
    const LINKED: &str = "/tmp/gate/x";

    #[test]
    fn the_main_worktree_comes_first_and_linked_ones_are_found_below_a_parent() {
        let listed = Worktrees::from(vec![PathBuf::from(MAIN), PathBuf::from(LINKED)]);
        assert_eq!(listed.main(), Some(Path::new(MAIN)));
        assert_eq!(listed.below(Path::new(GATES)), [Path::new(LINKED)]);
    }
}
