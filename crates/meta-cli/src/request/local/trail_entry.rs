use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::byte_count::ByteCount;
use super::reason::Reason;
use super::recorded_worktree::RecordedWorktree;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change")]
pub enum TrailEntry {
    Created(RecordedWorktree),
    Removed { path: PathBuf, at: DateTime<Utc> },
    Freed { path: PathBuf, at: DateTime<Utc>, reason: Reason, bytes: ByteCount },
    Refused { path: PathBuf, at: DateTime<Utc>, reason: Reason },
    Closed { path: PathBuf, at: DateTime<Utc> },
    Moved { path: PathBuf, to: PathBuf, at: DateTime<Utc> },
}

impl TrailEntry {
    pub fn applied(&self, current: Vec<RecordedWorktree>) -> Vec<RecordedWorktree> {
        match self {
            Self::Created(worktree) => self.dropped(current).chain([worktree.clone()]).collect(),
            Self::Removed { .. } | Self::Freed { .. } | Self::Closed { .. } => self.dropped(current).collect(),
            Self::Refused { .. } => current,
            Self::Moved { path, to, .. } => {
                current.into_iter().map(|worktree| Self::followed(worktree, path, to)).collect()
            }
        }
    }

    fn followed(worktree: RecordedWorktree, path: &Path, to: &Path) -> RecordedWorktree {
        if worktree.path() == path { worktree.relocated(to) } else { worktree }
    }

    fn dropped(&self, current: Vec<RecordedWorktree>) -> impl Iterator<Item = RecordedWorktree> + use<'_> {
        current.into_iter().filter(|worktree| worktree.path() != self.path())
    }

    fn path(&self) -> &Path {
        match self {
            Self::Created(worktree) => worktree.path(),
            Self::Removed { path, .. }
            | Self::Freed { path, .. }
            | Self::Refused { path, .. }
            | Self::Closed { path, .. }
            | Self::Moved { path, .. } => path,
        }
    }
}
