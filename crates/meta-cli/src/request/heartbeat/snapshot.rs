use std::time::Duration;

use chrono::{DateTime, Utc};
use meta_status::Flags;
use serde::{Deserialize, Serialize};

use super::gc_hook::GcHook;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    taken: DateTime<Utc>,
    flags: Flags,
    gc: GcHook,
}

impl Snapshot {
    pub fn new(taken: DateTime<Utc>, flags: Flags, gc: GcHook) -> Self {
        Self { taken, flags, gc }
    }

    pub fn taken(&self) -> DateTime<Utc> {
        self.taken
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    pub fn age(&self, now: DateTime<Utc>) -> Duration {
        (now - self.taken).to_std().unwrap_or_default()
    }
}
