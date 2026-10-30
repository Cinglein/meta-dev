use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entity::Task;
use crate::id::Id;
use crate::pull_request::PullRequest;
use crate::target::Target;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    Pull(PullRequest),
    Item(Target),
    Task(Id<Task>),
}

impl fmt::Display for Cause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pull(pull) => pull.fmt(formatter),
            Self::Item(item) => item.fmt(formatter),
            Self::Task(task) => task.fmt(formatter),
        }
    }
}
