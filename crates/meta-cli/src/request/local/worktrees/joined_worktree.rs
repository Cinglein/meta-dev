use std::fmt;

use meta_door::Door;
use meta_door::kinds::Kind;
use meta_state::{State, TaskStatus};

use crate::request::local::recorded_worktree::RecordedWorktree;

#[derive(Debug)]
pub struct JoinedWorktree {
    worktree: RecordedWorktree,
    status: Option<TaskStatus>,
}

impl JoinedWorktree {
    pub fn new(worktree: &RecordedWorktree, state: &State) -> Self {
        let task = state.tasks().iter().find(|task| task.new_task().task() == worktree.task());
        Self { worktree: worktree.clone(), status: task.map(|task| state.status_of(task)) }
    }
}

impl fmt::Display for JoinedWorktree {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = self.status.as_ref().map_or_else(|| "not in the log".to_owned(), ToString::to_string);
        let missing = if Door::kind(self.worktree.path()) == Kind::Missing { " missing" } else { "" };
        write!(formatter, "{} {status} {}{missing}", self.worktree.task(), self.worktree)
    }
}
