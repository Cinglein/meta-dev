use std::time::Duration;

use serde::Deserialize;

use super::seconds::Seconds;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "Seconds")]
pub struct SkewLimit(Duration);

impl SkewLimit {
    pub fn passed_by(self, ahead: Duration) -> Option<Duration> {
        (ahead > self.0).then_some(self.0)
    }
}

impl From<Seconds> for SkewLimit {
    fn from(seconds: Seconds) -> Self {
        Self(seconds.duration())
    }
}
