use std::fmt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use meta_schema::{Id, Task};
use serde::{Deserialize, Serialize};

use crate::request::project::BranchName;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedWorktree {
    path: PathBuf,
    branch: BranchName,
    task: Id<Task>,
    created: DateTime<Utc>,
}

impl RecordedWorktree {
    pub fn new(path: PathBuf, task: Id<Task>) -> Self {
        let Ok(branch) = task.to_string().parse();
        Self { path, branch, task, created: Utc::now() }
    }

    pub fn path(&self) -> &Path {
        &self.path
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
