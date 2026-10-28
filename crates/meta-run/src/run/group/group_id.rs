use std::fmt;
use std::process::Child;

use crate::run::Run;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupId(u32);

impl GroupId {
    pub fn appended(self, kill: Run) -> Run {
        kill.arg(self.to_string())
    }
}

impl From<&Child> for GroupId {
    fn from(leader: &Child) -> Self {
        Self(leader.id())
    }
}

#[cfg(test)]
impl std::str::FromStr for GroupId {
    type Err = std::num::ParseIntError;

    fn from_str(leader: &str) -> Result<Self, Self::Err> {
        leader.trim().parse().map(Self)
    }
}

impl fmt::Display for GroupId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "-{}", self.0)
    }
}
