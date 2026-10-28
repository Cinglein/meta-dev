use std::fmt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use meta_schema::{Id, Task};
use serde::{Deserialize, Serialize};

use super::worktree_id::WorktreeId;
use crate::request::project::BranchName;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedWorktree {
    path: PathBuf,
    branch: BranchName,
    task: Id<Task>,
    created: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    worktree: Option<WorktreeId>,
}

impl RecordedWorktree {
    pub fn new(path: PathBuf, task: Id<Task>) -> Self {
        Self { path, branch: BranchName::from(&task), task, created: Utc::now(), worktree: None }
    }

    pub fn identified(self, worktree: WorktreeId) -> Self {
        Self { worktree: Some(worktree), ..self }
    }

    pub fn relocated(self, path: &Path) -> Self {
        Self { path: path.to_path_buf(), ..self }
    }

    pub fn worktree(&self) -> Option<&WorktreeId> {
        self.worktree.as_ref()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn branch(&self) -> &BranchName {
        &self.branch
    }

    pub fn task(&self) -> Id<Task> {
        self.task
    }
}

impl fmt::Display for RecordedWorktree {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let created = self.created.format("%Y-%m-%d %H:%MZ");
        write!(formatter, "{} {} created {created}", self.branch, self.path.display())
    }
}
