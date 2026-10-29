use std::fmt;

use serde::{Deserialize, Serialize};

use super::command_summary::CommandSummary;
use super::process_row::{CommandLine, Pid};
use super::target::Target;
use crate::request::local::local_settings::Cause;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Culprit {
    pid: Pid,
    #[serde(skip)]
    command: CommandLine,
    summary: CommandSummary,
    cause: Cause,
    target: Target,
}

impl Culprit {
    pub fn new(pid: Pid, command: CommandLine, cause: Cause, target: Target) -> Self {
        Self { pid, summary: CommandSummary::from(&command), command, cause, target }
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }

    pub fn target(&self) -> Target {
        self.target
    }

    pub fn cause(&self) -> Cause {
        self.cause
    }

    pub fn names(&self, pid: Pid, command: &CommandLine) -> Option<()> {
        (self.pid == pid && self.command == *command).then_some(())
    }

    pub fn repeats(&self, other: &Self) -> Option<()> {
        (self.pid == other.pid && self.summary == other.summary).then_some(())
    }
}

impl fmt::Display for Culprit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "pid {} {}, target {}: {}", self.pid, self.cause, self.target, self.summary)
    }
}
