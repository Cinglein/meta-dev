use std::fmt;
use std::num::NonZeroU64;
use std::time::Duration;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(transparent)]
pub struct Seconds(NonZeroU64);

impl Seconds {
    pub fn duration(self) -> Duration {
        Duration::from_secs(self.0.get())
    }
}

impl fmt::Display for Seconds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
