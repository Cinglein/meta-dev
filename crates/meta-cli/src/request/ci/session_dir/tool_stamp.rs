use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::request::ci::task_worktree::TaskWorktree;
use crate::request::hook::CallPhase;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolStamp {
    worktree: TaskWorktree,
    at: DateTime<Utc>,
    call: CallPhase,
}

impl ToolStamp {
    pub fn new(worktree: TaskWorktree, at: DateTime<Utc>, call: CallPhase) -> Self {
        Self { worktree, at, call }
    }

    pub fn worktree(&self) -> &TaskWorktree {
        &self.worktree
    }

    pub fn call(&self) -> CallPhase {
        self.call
    }

    pub fn quiet(&self, now: DateTime<Utc>) -> Duration {
        (now - self.at).to_std().unwrap_or_default()
    }

    pub fn newer_than(&self, other: &Self) -> Option<()> {
        (self.at > other.at).then_some(())
    }
}
