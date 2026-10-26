use std::fmt;
use std::time::Duration;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct Millis(u64);

impl Millis {
    pub fn duration(self) -> Duration {
        Duration::from_millis(self.0)
    }
}

impl fmt::Display for Millis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}ms", self.0)
    }
}
