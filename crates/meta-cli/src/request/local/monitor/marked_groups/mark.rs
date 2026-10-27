use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::request::local::local_settings::MonitorSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    started: DateTime<Utc>,
    seen: DateTime<Utc>,
}

impl Mark {
    const START_SLACK: TimeDelta = TimeDelta::seconds(2);

    pub fn new(started: DateTime<Utc>, seen: DateTime<Utc>) -> Self {
        Self { started, seen }
    }

    pub fn kept(self, leader: Option<DateTime<Utc>>, now: DateTime<Utc>, settings: &MonitorSettings) -> Option<Self> {
        settings.mark_fresh(self.seen, self.seen)?;
        let same = |leader: DateTime<Utc>| (leader - self.started).abs() <= Self::START_SLACK;
        (leader.is_none_or(same) || leader.is_some()).then_some(Self { seen: now, ..self })
    }
}
