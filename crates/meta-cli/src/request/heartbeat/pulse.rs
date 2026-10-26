use chrono::{DateTime, Utc};

use super::gc_hook::GcHook;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pulse {
    now: DateTime<Utc>,
    gc: GcHook,
}

impl Pulse {
    pub fn new(now: DateTime<Utc>, gc: GcHook) -> Self {
        Self { now, gc }
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.now
    }

    pub fn gc_hook(self) -> GcHook {
        self.gc
    }
}
