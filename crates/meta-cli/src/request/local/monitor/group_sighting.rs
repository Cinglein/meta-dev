use chrono::{DateTime, Utc};

use super::marked_groups::Mark;
use super::process_row::GroupId;
use crate::request::local::local_settings::MonitorSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupSighting {
    group: GroupId,
    started: DateTime<Utc>,
    leader: Option<DateTime<Utc>>,
    wrapped: Option<()>,
}

impl GroupSighting {
    pub fn new(group: GroupId, started: DateTime<Utc>, leader: Option<DateTime<Utc>>, wrapped: Option<()>) -> Self {
        Self { group, started, leader, wrapped }
    }

    pub fn group(&self) -> GroupId {
        self.group
    }

    pub fn marked(&self, before: Option<Mark>, now: DateTime<Utc>, settings: &MonitorSettings) -> Option<Mark> {
        let fresh = self.wrapped.map(|()| Mark::new(self.leader.unwrap_or(self.started), now));
        fresh.or_else(|| before?.kept(self.leader, now, settings))
    }
}
