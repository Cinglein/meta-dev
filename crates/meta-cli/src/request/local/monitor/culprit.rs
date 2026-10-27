use std::fmt;

use serde::{Deserialize, Serialize};

use super::process_row::{CommandLine, Pid};
use super::target::Target;
use crate::request::local::local_settings::Cause;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Culprit {
    pid: Pid,
    command: CommandLine,
    cause: Cause,
    target: Target,
}

impl Culprit {
    pub fn new(pid: Pid, command: CommandLine, cause: Cause, target: Target) -> Self {
        Self { pid, command, cause, target }
    }

    pub fn target(&self) -> Target {
        self.target
    }

    pub fn command(&self) -> &CommandLine {
        &self.command
    }
}

impl fmt::Display for Culprit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "pid {} {}, target {}: {}", self.pid, self.cause, self.target, self.command)
    }
}
