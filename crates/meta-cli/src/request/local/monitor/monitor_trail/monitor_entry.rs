use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use meta_schema::DueTime;
use serde::{Deserialize, Serialize};

use super::super::finding::Finding;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorEntry {
    at: DateTime<Utc>,
    finding: Finding,
}

impl MonitorEntry {
    pub fn new(at: DateTime<Utc>, finding: Finding) -> Self {
        Self { at, finding }
    }

    pub fn age(&self, now: DateTime<Utc>) -> Duration {
        (now - self.at).to_std().unwrap_or_default()
    }
}

impl fmt::Display for MonitorEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", DueTime::from(self.at), self.finding)
    }
}
