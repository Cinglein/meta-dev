use std::fmt;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixSeconds(u64);

impl From<Duration> for UnixSeconds {
    fn from(since_epoch: Duration) -> Self {
        Self(since_epoch.as_secs())
    }
}

impl FromStr for UnixSeconds {
    type Err = std::num::ParseIntError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.trim().parse().map(Self)
    }
}

impl fmt::Display for UnixSeconds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
