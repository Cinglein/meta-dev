use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entity::Task;
use crate::id::Id;
use crate::pull_request::PullRequest;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskLink {
    task: Id<Task>,
    pr: PullRequest,
}

impl TaskLink {
    #[must_use]
    pub fn new(task: Id<Task>, pr: PullRequest) -> Self {
        Self { task, pr }
    }

    #[must_use]
    pub fn task(&self) -> Id<Task> {
        self.task
    }

    #[must_use]
    pub fn pull_request(&self) -> PullRequest {
        self.pr
    }
}

impl fmt::Display for TaskLink {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.task, self.pr)
    }
}
