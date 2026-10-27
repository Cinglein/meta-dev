use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpectedRun(Duration);

impl ExpectedRun {
    pub fn duration(self) -> Duration {
        self.0
    }
}

impl From<Duration> for ExpectedRun {
    fn from(took: Duration) -> Self {
        Self(took)
    }
}

impl FromStr for ExpectedRun {
    type Err = ParseIntError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.trim().parse().map(|millis| Self(Duration::from_millis(millis)))
    }
}

impl fmt::Display for ExpectedRun {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.as_millis())
    }
}
