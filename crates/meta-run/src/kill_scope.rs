pub(crate) mod kill_target;

use std::os::unix::process::CommandExt as _;
use std::process::Command;

use crate::leader::Leader;
use kill_target::KillTarget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KillScope {
    Process,
    Group,
}

impl KillScope {
    pub(crate) fn led(self, command: &mut Command) -> &mut Command {
        match self {
            Self::Process => command,
            Self::Group => command.process_group(i32::default()),
        }
    }

    pub(crate) fn target(self, leader: Leader) -> KillTarget {
        match self {
            Self::Process => KillTarget::Process(leader),
            Self::Group => KillTarget::Group(leader),
        }
    }
}
