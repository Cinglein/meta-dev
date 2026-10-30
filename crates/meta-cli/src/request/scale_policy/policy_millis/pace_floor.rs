use std::num::NonZeroU64;
use std::time::Duration;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct PaceFloor(NonZeroU64);

impl PaceFloor {
    pub fn duration(self) -> Duration {
        Duration::from_millis(self.0.get())
    }
}
