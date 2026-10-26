use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    Green,
    Pending,
    Red,
}

impl Verdict {
    pub fn failing(self) -> Option<Self> {
        match self {
            Self::Red => Some(self),
            Self::Green | Self::Pending => None,
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Green => "green",
            Self::Pending => "pending",
            Self::Red => "red",
        })
    }
}
