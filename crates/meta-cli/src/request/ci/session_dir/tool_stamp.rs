use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::request::ci::task_worktree::TaskWorktree;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolStamp {
    worktree: TaskWorktree,
    at: DateTime<Utc>,
}

impl ToolStamp {
    pub fn new(worktree: TaskWorktree, at: DateTime<Utc>) -> Self {
        Self { worktree, at }
    }

    pub fn worktree(&self) -> &TaskWorktree {
        &self.worktree
    }

    pub fn quiet(&self, now: DateTime<Utc>) -> Duration {
        (now - self.at).to_std().unwrap_or_default()
    }
}
