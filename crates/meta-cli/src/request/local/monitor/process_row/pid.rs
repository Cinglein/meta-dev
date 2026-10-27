use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::super::unreadable::Unreadable;
use super::ProcessRow;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pid(u32);

impl Pid {
    pub const INIT: Self = Self(1);

    pub fn ancestry(self, rows: &[ProcessRow]) -> Vec<Self> {
        let parent = |pid: &Self| rows.iter().find_map(|row| row.parent_of(*pid));
        std::iter::successors(Some(self), |pid| parent(pid).filter(|parent| parent != pid)).take(rows.len()).collect()
    }
}

impl From<u32> for Pid {
    fn from(pid: u32) -> Self {
        Self(pid)
    }
}

impl FromStr for Pid {
    type Err = Unreadable;

    fn from_str(pid: &str) -> Result<Self, Self::Err> {
        pid.trim().parse().map(Self).map_err(|_| Unreadable)
    }
}

impl fmt::Display for Pid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
