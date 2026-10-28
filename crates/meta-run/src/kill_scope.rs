mod leader;

use std::os::unix::process::CommandExt as _;
use std::process::{Child, Command};

use leader::Leader;

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

    pub(crate) fn ended(self, child: &mut Child) {
        let group = match self {
            Self::Process => None,
            Self::Group => Some(Leader::from(&*child)),
        };
        group.into_iter().for_each(Leader::ended);
        drop(child.kill());
        drop(child.wait());
    }
}
