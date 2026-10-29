use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Line(usize);

impl Line {
    #[must_use]
    pub fn following(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl FromStr for Line {
    type Err = ParseIntError;

    fn from_str(range: &str) -> Result<Self, Self::Err> {
        range.split(',').next().unwrap_or_default().parse().map(Self)
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
