use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::super::unreadable::Unreadable;
use super::pid::Pid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(Pid);

impl GroupId {
    pub fn leader(self) -> Pid {
        self.0
    }
}

impl From<Pid> for GroupId {
    fn from(leader: Pid) -> Self {
        Self(leader)
    }
}

impl FromStr for GroupId {
    type Err = Unreadable;

    fn from_str(group: &str) -> Result<Self, Self::Err> {
        group.parse().map(Self)
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
