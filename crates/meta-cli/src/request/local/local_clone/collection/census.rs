use std::path::{Path, PathBuf};

use super::tracked::Tracked;

#[derive(Clone, Debug)]
pub struct Census {
    tracked: Vec<Tracked>,
    listed: Vec<PathBuf>,
}

impl Census {
    pub fn new(tracked: Vec<Tracked>, listed: Vec<PathBuf>) -> Self {
        Self { tracked, listed }
    }

    pub fn tracked(&self) -> &[Tracked] {
        &self.tracked
    }

    pub fn unrecorded(&self) -> impl Iterator<Item = &Path> {
        let recorded = |path: &&PathBuf| self.tracked.iter().any(|tracked| tracked.worktree().path() == path.as_path());
        self.listed.iter().filter(move |path| !recorded(path)).map(PathBuf::as_path)
    }
}
