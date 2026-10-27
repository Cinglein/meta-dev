use std::num::NonZeroU64;
use std::time::Duration;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "NonZeroU64")]
pub struct Limit(Duration);

impl Limit {
    pub fn passed_by(self, spent: Duration) -> Option<()> {
        (spent > self.0).then_some(())
    }

    pub fn duration(self) -> Duration {
        self.0
    }
}

impl From<NonZeroU64> for Limit {
    fn from(seconds: NonZeroU64) -> Self {
        Self(Duration::from_secs(seconds.get()))
    }
}
