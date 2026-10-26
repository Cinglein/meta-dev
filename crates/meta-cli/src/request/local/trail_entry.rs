use std::path::PathBuf;

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
        match self {
            Self::Created(worktree) => [current, vec![worktree]].concat(),
            Self::Removed { path, .. } => current.into_iter().filter(|worktree| worktree.path() != path).collect(),
        }
    }
}
