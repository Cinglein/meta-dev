use std::fmt;

use meta_gh::Outcome;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrunkVerdict {
    Green,
    Red,
    Unknown,
}

impl TrunkVerdict {
    #[must_use]
    pub fn decided(outcome: Outcome) -> Option<Self> {
        outcome.decided(Self::Green, Self::Red)
    }
}

impl fmt::Display for TrunkVerdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Green => "green",
            Self::Red => "red",
            Self::Unknown => "unknown",
        })
    }
}
