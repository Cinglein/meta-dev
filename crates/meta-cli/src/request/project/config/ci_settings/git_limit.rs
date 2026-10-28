use std::time::{Duration, Instant};

use serde::Deserialize;

use super::millis::Millis;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(from = "Millis")]
pub struct GitLimit(Duration);

impl GitLimit {
    pub fn deadline(self) -> Instant {
        Instant::now() + self.0
    }
}

impl From<Millis> for GitLimit {
    fn from(millis: Millis) -> Self {
        Self(millis.duration())
    }
}
