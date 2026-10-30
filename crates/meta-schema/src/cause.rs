use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::Task;
use crate::id::Id;
use crate::pull_request::PullRequest;
use crate::target::Target;
use crate::unrecognised::Unrecognised;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    Pull(PullRequest),
    Item(Target),
    Task(Id<Task>),
}

impl FromStr for Cause {
    type Err = Unrecognised;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.parse()
            .map(Self::Item)
            .or_else(|_| text.parse().map(Self::Task))
            .or_else(|_| text.parse().map(Self::Pull))
            .map_err(|_| Unrecognised::new("a pull request number, concern id, issue id or task id", text))
    }
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
