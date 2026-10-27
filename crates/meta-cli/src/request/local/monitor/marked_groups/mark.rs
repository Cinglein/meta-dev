use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::request::local::local_settings::MonitorSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    started: DateTime<Utc>,
    seen: DateTime<Utc>,
}

impl Mark {
    pub fn new(started: DateTime<Utc>, seen: DateTime<Utc>) -> Self {
        Self { started, seen }
    }

    pub fn fits(
        self,
        leader: Option<DateTime<Utc>>,
        earliest: DateTime<Utc>,
        settings: &MonitorSettings,
    ) -> Option<()> {
        let slack = settings.start_slack();
        let fits = match leader {
            Some(leader) => (leader - self.started).abs() <= slack,
            None => earliest <= self.seen + slack || earliest > self.seen,
        };
        fits.then_some(())
    }

    pub fn refreshed(self, now: DateTime<Utc>, settings: &MonitorSettings) -> Option<Self> {
        settings.mark_fresh(self.seen, now).map(|()| Self { seen: now, ..self })
    }
}
