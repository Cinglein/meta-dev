use std::num::NonZeroU64;
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use meta_schema::DueTime;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct After(NonZeroU64);

impl After {
    pub fn duration(self) -> Duration {
        Duration::from_millis(self.0.get())
    }

    pub fn due_after(self, now: DateTime<Utc>) -> Option<DueTime> {
        let within = TimeDelta::from_std(self.duration()).ok()?;
        now.checked_add_signed(within).map(DueTime::from)
    }
}
