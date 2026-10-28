use std::fmt;
use std::num::ParseIntError;
use std::process::Child;
use std::str::FromStr;

use crate::run::Run;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(u32);

impl Pid {
    pub fn appended(self, run: Run) -> Run {
        run.arg(self.to_string())
    }
}

impl From<&Child> for Pid {
    fn from(child: &Child) -> Self {
        Self(child.id())
    }
}

impl FromStr for Pid {
    type Err = ParseIntError;

    fn from_str(pid: &str) -> Result<Self, Self::Err> {
        pid.parse().map(Self)
    }
}

impl fmt::Display for Pid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
