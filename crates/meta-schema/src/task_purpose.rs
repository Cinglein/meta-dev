use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entity::Task;
use crate::id::Id;
use crate::purpose::Purpose;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskPurpose {
    task: Id<Task>,
    purpose: Purpose,
}

impl TaskPurpose {
    #[must_use]
    pub fn task(&self) -> Id<Task> {
        self.task
    }
}

impl fmt::Display for TaskPurpose {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.task, self.purpose)
    }
}
