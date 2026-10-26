use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::recorded_worktree::RecordedWorktree;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change")]
pub enum TrailEntry {
    Created(RecordedWorktree),
    Removed { path: PathBuf, at: DateTime<Utc> },
}

impl TrailEntry {
    pub fn applied<'trail>(&'trail self, current: Vec<&'trail RecordedWorktree>) -> Vec<&'trail RecordedWorktree> {
        let kept = current.into_iter().filter(|worktree| worktree.path() != self.path());
        match self {
            Self::Created(worktree) => kept.chain([worktree]).collect(),
            Self::Removed { .. } => kept.collect(),
        }
    }

    fn path(&self) -> &Path {
        match self {
            Self::Created(worktree) => worktree.path(),
            Self::Removed { path, .. } => path,
        }
    }
}
