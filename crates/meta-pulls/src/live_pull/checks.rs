use std::fmt;

use crate::verdict::Verdict;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checks {
    Required(Verdict),
    Unrequired(Verdict),
    Truncated,
    Absent,
}

impl Checks {
    pub fn failing(self) -> Option<Verdict> {
        match self {
            Self::Required(verdict) | Self::Unrequired(verdict) => verdict.failing(),
            Self::Truncated | Self::Absent => None,
        }
    }

    pub fn verdict(self) -> Verdict {
        match self {
            Self::Required(verdict) | Self::Unrequired(verdict) => verdict,
            Self::Truncated | Self::Absent => Verdict::Pending,
        }
    }

    pub fn truncated(self) -> Option<()> {
        match self {
            Self::Truncated => Some(()),
            Self::Required(_) | Self::Unrequired(_) | Self::Absent => None,
        }
    }
}

impl fmt::Display for Checks {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Required(verdict) => write!(formatter, "required {verdict}"),
            Self::Unrequired(verdict) => write!(formatter, "checks {verdict} (none required)"),
            Self::Truncated => formatter.write_str("checks truncated"),
            Self::Absent => formatter.write_str("no checks"),
        }
    }
}
