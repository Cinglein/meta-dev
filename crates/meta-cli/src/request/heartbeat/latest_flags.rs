mod heartbeat_flag;

use std::fmt;
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Utc};
use meta_schema::DueTime;
use meta_status::Flags;

use super::heartbeat_record::HeartbeatRecord;
use super::snapshot::Snapshot;
use crate::request::cli_failure::CliFailure;
use crate::request::output::{Listed, Output};
use crate::request::project::HeartbeatSettings;
use heartbeat_flag::HeartbeatFlag;

#[derive(Debug)]
pub struct LatestFlags {
    seen: Option<Snapshot>,
    age: Duration,
    stale: Option<HeartbeatFlag>,
}

impl LatestFlags {
    pub fn answer(git_dir: &Path, settings: &HeartbeatSettings, now: DateTime<Utc>) -> Result<Output, CliFailure> {
        let record = HeartbeatRecord::read(git_dir)?;
        Ok(record.map_or_else(Self::never_ticked, |record| Self::seen(record.latest().clone(), settings, now)).output())
    }

    fn never_ticked() -> Self {
        Self { seen: None, age: Duration::ZERO, stale: Some(HeartbeatFlag::NeverTicked) }
    }

    fn seen(snapshot: Snapshot, settings: &HeartbeatSettings, now: DateTime<Utc>) -> Self {
        let age = snapshot.age(now);
        let skewed = snapshot.ahead(now).and_then(|ahead| {
            settings.skew_limit().passed_by(ahead).map(|limit| HeartbeatFlag::ClockSkew { ahead, limit })
        });
        let stale = skewed
            .or_else(|| settings.stale_limit().passed_by(age).map(|limit| HeartbeatFlag::Stale { age, limit }))
            .or_else(|| settings.gc_mode().sweep().map_or(Some(HeartbeatFlag::GcDisabled), |_| None));
        Self { seen: Some(snapshot), age, stale }
    }

    fn flags(&self) -> Flags {
        self.seen.as_ref().map(|snapshot| snapshot.flags().clone()).unwrap_or_default()
    }

    fn output(self) -> Output {
        let stale = self.stale.map(|_| ());
        match stale.or_else(|| self.flags().awaiting_judgement().raised().map(|_| ())) {
            Some(()) => Output::Flagged(Listed::new(self)),
            None => Output::Listed(Listed::new(self)),
        }
    }
}

impl fmt::Display for LatestFlags {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.seen.iter().try_for_each(|snapshot| {
            writeln!(formatter, "heartbeat {}, {}s ago", DueTime::from(snapshot.taken()), self.age.as_secs())
        })?;
        let flags = self.flags();
        match (self.stale, flags.raised()) {
            (None, None) => writeln!(formatter, "flags: none"),
            (stale, _) => {
                writeln!(formatter, "flags:")?;
                stale.iter().try_for_each(|stale| write!(formatter, "{stale}"))?;
                write!(formatter, "{flags}")
            }
        }
    }
}
