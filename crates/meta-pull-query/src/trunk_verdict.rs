use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrunkVerdict {
    Green,
    Red,
    Unknown,
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
