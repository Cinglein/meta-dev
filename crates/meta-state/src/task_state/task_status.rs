use std::fmt;

use meta_schema::{ExternalKind, Id, Issue, Task};

use crate::stamp::Stamp;
use crate::task_state::stalling::Stalling;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Open,
    Assigned,
    Discussion(Id<Issue>),
    WaitingOnTask(Id<Task>),
    WaitingOnExternal(ExternalKind),
    Done(Stamp),
    Retired,
}

impl TaskStatus {
    #[must_use]
    pub fn stalling(&self) -> Stalling {
        match self {
            Self::Open | Self::Assigned => Stalling::Possible,
            Self::Discussion(_)
            | Self::WaitingOnTask(_)
            | Self::WaitingOnExternal(_)
            | Self::Done(_)
            | Self::Retired => Stalling::Never,
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open => formatter.write_str("open"),
            Self::Assigned => formatter.write_str("assigned"),
            Self::Discussion(issue) => write!(formatter, "in discussion {issue}"),
            Self::WaitingOnTask(task) => write!(formatter, "waiting on {task}"),
            Self::WaitingOnExternal(kind) => write!(formatter, "waiting on {kind}"),
            Self::Done(stamp) => write!(formatter, "done by {stamp}"),
            Self::Retired => formatter.write_str("retired"),
        }
    }
}
