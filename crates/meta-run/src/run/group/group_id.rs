use std::fmt;
use std::process::Child;

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupId(u32);

impl GroupId {
    pub fn signalled(self, signal: Signal) {
        let _delivered = self.pid().map(|group| killpg(group, signal));
    }

    pub fn alive(self) -> bool {
        self.pid().is_some_and(|group| killpg(group, None).is_ok())
    }

    fn pid(self) -> Option<Pid> {
        i32::try_from(self.0).ok().map(Pid::from_raw)
    }
}

impl From<&Child> for GroupId {
    fn from(leader: &Child) -> Self {
        Self(leader.id())
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "-{}", self.0)
    }
}
