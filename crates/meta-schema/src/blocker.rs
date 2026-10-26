use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::blocker_visitor::BlockerVisitor;
use crate::entity::{Issue, Task};
use crate::external_kind::ExternalKind;
use crate::id::Id;
use crate::unrecognised::Unrecognised;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Blocker {
    Discussion(Id<Issue>),
    Task(Id<Task>),
    External(ExternalKind),
}

impl Blocker {
    pub fn visit<Visitor: BlockerVisitor>(&self, visitor: Visitor) -> Visitor::Output {
        match self {
            Self::Discussion(issue) => visitor.discussion(*issue),
            Self::Task(task) => visitor.task(*task),
            Self::External(kind) => visitor.external(*kind),
        }
    }
}

impl FromStr for Blocker {
    type Err = Unrecognised;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.parse()
            .map(Self::Discussion)
            .or_else(|_| text.parse().map(Self::Task))
            .or_else(|_| text.parse().map(Self::External))
            .map_err(|_| Unrecognised::new("an issue id, a task id, review, ci or merge", text))
    }
}

impl fmt::Display for Blocker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discussion(issue) => issue.fmt(formatter),
            Self::Task(task) => task.fmt(formatter),
            Self::External(kind) => kind.fmt(formatter),
        }
    }
}
