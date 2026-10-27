use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use meta_schema::DueTime;
use serde::{Deserialize, Serialize};

use super::command_summary::CommandSummary;
use super::process_row::Pid;
use crate::request::local::local_settings::TickCount;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchItem {
    pid: Pid,
    summary: CommandSummary,
    first: DateTime<Utc>,
    last: DateTime<Utc>,
    ticks: TickCount,
    cpu_time: Duration,
    before: Duration,
}

impl WatchItem {
    pub fn new(pid: Pid, summary: CommandSummary, now: DateTime<Utc>, cpu_time: Duration) -> Self {
        let ticks = TickCount::default().next();
        Self { pid, summary, first: now, last: now, ticks, cpu_time, before: cpu_time }
    }

    #[must_use]
    pub fn followed(self, before: Option<&Self>) -> Self {
        before.and_then(|before| self.same(before).map(|()| before)).map_or(self.clone(), |before| Self {
            first: before.first,
            ticks: before.ticks.next(),
            before: before.cpu_time,
            ..self
        })
    }

    pub fn same(&self, other: &Self) -> Option<()> {
        (self.pid == other.pid && self.summary == other.summary).then_some(())
    }

    pub fn ticks(&self) -> TickCount {
        self.ticks
    }
}

impl fmt::Display for WatchItem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let trend = if self.cpu_time > self.before { "rising" } else { "flat" };
        write!(
            formatter,
            "pid {} {}: cpu {}s {trend}, {} ticks since {}, last {}",
            self.pid,
            self.summary,
            self.cpu_time.as_secs(),
            self.ticks,
            DueTime::from(self.first),
            DueTime::from(self.last)
        )
    }
}
