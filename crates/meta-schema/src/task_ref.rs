use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entity::Task;
use crate::id::Id;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRef {
    task: Id<Task>,
}

impl TaskRef {
    pub fn new(task: Id<Task>) -> Self {
        Self { task }
    }

    pub fn task(&self) -> Id<Task> {
        self.task
    }
}

impl fmt::Display for TaskRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.task.fmt(formatter)
    }
}
