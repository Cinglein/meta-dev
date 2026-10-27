use std::fmt;
use std::process::Command;

use serde::{Deserialize, Serialize};

use super::process_row::Pid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Target {
    Group(Pid),
    Process(Pid),
}

impl Target {
    pub fn covers(self, pid: Pid, group: Pid, same_command: Option<()>) -> Option<()> {
        match self {
            Self::Group(led) => (led == group).then_some(()),
            Self::Process(alone) => same_command.filter(|()| alone == pid),
        }
    }

    pub fn appended(self, command: &mut Command) {
        command.arg(self.to_string());
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Group(group) => write!(formatter, "-{group}"),
            Self::Process(pid) => write!(formatter, "{pid}"),
        }
    }
}
