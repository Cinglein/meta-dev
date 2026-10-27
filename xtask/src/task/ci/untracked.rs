use meta_schema::{Id, Task};
use task::BranchName;
use thiserror::Error;

use crate::task::failure::Failure;

#[derive(Debug, Error)]
pub enum Untracked {
    #[error("pull request head branch unknown")]
    NoBranch,
    #[error("branch {0} names no task (task-<id> or task-<id>-<suffix>)")]
    NotTask(BranchName),
    #[error("cannot read meta/log ({0})")]
    Unreadable(Box<Failure>),
    #[error("{0} is not in meta/log")]
    Unknown(Id<Task>),
    #[error("{0} is retired")]
    Retired(Id<Task>),
    #[error("{0} has no assignee")]
    Unclaimed(Id<Task>),
}
