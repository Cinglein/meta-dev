use std::time::Duration;

use chrono::{DateTime, Utc};
use meta_status::Flags;
use serde::{Deserialize, Serialize};

use super::gc_hook::GcHook;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    at: DateTime<Utc>,
    flags: Flags,
    gc: GcHook,
}

impl Snapshot {
    pub fn new(at: DateTime<Utc>, flags: Flags, gc: GcHook) -> Self {
        Self { at, flags, gc }
    }

    pub fn at(&self) -> DateTime<Utc> {
        self.at
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    pub fn age(&self, now: DateTime<Utc>) -> Duration {
        (now - self.at).to_std().unwrap_or_default()
    }
}
