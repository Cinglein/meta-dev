use std::time::Duration;

use chrono::{DateTime, Utc};

use super::gc_hook::GcHook;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pulse {
    now: DateTime<Utc>,
    gc: GcHook,
    ci_run_within: Duration,
}

impl Pulse {
    pub fn new(now: DateTime<Utc>, gc: GcHook, ci_run_within: Duration) -> Self {
        Self { now, gc, ci_run_within }
    }

    pub fn ci_run_within(&self) -> Duration {
        self.ci_run_within
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.now
    }

    pub fn gc_hook(self) -> GcHook {
        self.gc
    }
}
