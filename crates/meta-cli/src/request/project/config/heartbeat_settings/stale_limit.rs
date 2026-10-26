use std::time::Duration;

use serde::Deserialize;

use super::seconds::Seconds;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "Seconds")]
pub struct StaleLimit(Duration);

impl StaleLimit {
    pub fn over(self, interval: Seconds) -> Option<()> {
        (self.0 > interval.duration()).then_some(())
    }

    pub fn passed_by(self, age: Duration) -> Option<Duration> {
        (age > self.0).then_some(self.0)
    }
}

impl From<Seconds> for StaleLimit {
    fn from(seconds: Seconds) -> Self {
        Self(seconds.duration())
    }
}
