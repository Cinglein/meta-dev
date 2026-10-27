use std::time::{Duration, Instant};

use serde::Deserialize;

use super::seconds::Seconds;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "Seconds")]
pub struct GcLimit(Duration);

impl GcLimit {
    pub fn deadline(self) -> Instant {
        Instant::now() + self.0
    }
}

impl From<Seconds> for GcLimit {
    fn from(seconds: Seconds) -> Self {
        Self(seconds.duration())
    }
}
