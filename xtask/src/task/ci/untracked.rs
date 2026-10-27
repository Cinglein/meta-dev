use meta_schema::{Id, Task};
use task::BranchName;
use thiserror::Error;

use crate::task::failure::Failure;

#[derive(Debug, Error)]
pub enum Untracked {
    #[error("pull request head branch unknown; {OPEN}", OPEN = Untracked::OPEN)]
    NoBranch,
    #[error("branch {0} names no task (task-<id>, task-<id>-<suffix> or task-<id>/<suffix>); {OPEN}", OPEN = Untracked::OPEN)]
    NotTask(BranchName),
    #[error("cannot read meta/log ({0}); bump meta-xtask / merge main, then rebuild meta")]
    Unreadable(Box<Failure>),
    #[error("{0} is not in meta/log; {OPEN}", OPEN = Untracked::OPEN)]
    Unknown(Id<Task>),
    #[error("{0} is retired; {OPEN}", OPEN = Untracked::OPEN)]
    Retired(Id<Task>),
    #[error("{0} has no assignee; meta claim {0}")]
    Unclaimed(Id<Task>),
}

impl Untracked {
    const OPEN: &str = "meta open \"<title>\" --division <path>, then meta claim <task> --worktree, then work on the branch that claim makes";
}
