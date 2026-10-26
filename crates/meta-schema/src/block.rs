use std::fmt;

use serde::{Deserialize, Serialize};

use crate::blocker::Blocker;
use crate::entity::Task;
use crate::id::Id;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    task: Id<Task>,
    on: Blocker,
}

impl Block {
    #[must_use]
    pub fn new(task: Id<Task>, on: Blocker) -> Self {
        Self { task, on }
    }

    #[must_use]
    pub fn task(&self) -> Id<Task> {
        self.task
    }

    #[must_use]
    pub fn blocker(&self) -> Blocker {
        self.on
    }
}

impl fmt::Display for Block {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} on {}", self.task, self.on)
    }
}
