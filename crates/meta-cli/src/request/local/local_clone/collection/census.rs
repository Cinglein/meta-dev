use std::path::{Path, PathBuf};

use super::tracked::Tracked;
use crate::request::local::local_clone::quarantine::Quarantine;

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
        let recorded = |path: &Path| self.tracked.iter().any(|tracked| tracked.worktree().path() == path);
        let claimed =
            move |path: &&PathBuf| recorded(path) || Quarantine::origin(path).is_some_and(|origin| recorded(&origin));
        self.listed.iter().filter(move |path| !claimed(path)).map(PathBuf::as_path)
    }

    pub fn quarantine_of(&self, path: &Path) -> Option<&Path> {
        self.listed.iter().find(|listed| Quarantine::origin(listed).as_deref() == Some(path)).map(PathBuf::as_path)
    }
}
