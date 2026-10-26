use std::fmt;

use super::verdict::Verdict;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checks {
    Required(Verdict),
    Unrequired(Verdict),
    Absent,
}

impl Checks {
    pub fn failing(self) -> Option<Verdict> {
        match self {
            Self::Required(verdict) | Self::Unrequired(verdict) => verdict.failing(),
            Self::Absent => None,
        }
    }
}

impl fmt::Display for Checks {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Required(verdict) => write!(formatter, "required {verdict}"),
            Self::Unrequired(verdict) => write!(formatter, "checks {verdict} (none required)"),
            Self::Absent => formatter.write_str("no checks"),
        }
    }
}
