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
}

impl TrailEntry {
    pub fn applied<'trail>(&'trail self, current: Vec<&'trail RecordedWorktree>) -> Vec<&'trail RecordedWorktree> {
        match self {
            Self::Created(worktree) => self.dropped(current).chain([worktree]).collect(),
            Self::Removed { .. } | Self::Freed { .. } | Self::Closed { .. } => self.dropped(current).collect(),
            Self::Refused { .. } => current,
        }
    }

    fn dropped<'trail>(
        &self,
        current: Vec<&'trail RecordedWorktree>,
    ) -> impl Iterator<Item = &'trail RecordedWorktree> + use<'trail, '_> {
        current.into_iter().filter(|worktree| worktree.path() != self.path())
    }

    fn path(&self) -> &Path {
        match self {
            Self::Created(worktree) => worktree.path(),
            Self::Removed { path, .. }
            | Self::Freed { path, .. }
            | Self::Refused { path, .. }
            | Self::Closed { path, .. } => path,
        }
    }
}
