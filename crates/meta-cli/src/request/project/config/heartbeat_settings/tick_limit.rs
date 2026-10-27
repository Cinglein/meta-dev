use std::fmt;
use std::time::{Duration, Instant};

use serde::Deserialize;

use super::seconds::Seconds;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "Seconds")]
pub struct TickLimit(Duration);

impl TickLimit {
    pub fn deadline(self) -> Instant {
        Instant::now() + self.0
    }

    pub fn under(self, interval: Seconds) -> Option<()> {
        (self.0 < interval.duration()).then_some(())
    }
}

impl From<Seconds> for TickLimit {
    fn from(seconds: Seconds) -> Self {
        Self(seconds.duration())
    }
}

impl From<Duration> for TickLimit {
    fn from(limit: Duration) -> Self {
        Self(limit)
    }
}

impl fmt::Display for TickLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}s", self.0.as_secs())
    }
}
