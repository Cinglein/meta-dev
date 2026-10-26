use std::path::Path;

use chrono::{DateTime, Utc};
use meta_log::LogStore;
use meta_pull_query::LiveQuery;
use meta_status::Github;

use super::gc_hook::GcHook;
use super::heartbeat_record::HeartbeatRecord;
use super::snapshot::Snapshot;
use crate::request::cli_failure::CliFailure;
use crate::request::output::{Listed, Output};
use crate::request::status_view::StatusView;

#[derive(Debug)]
pub struct Tick;

impl Tick {
    pub fn answer(
        git_dir: &Path,
        log: &impl LogStore,
        github: impl FnOnce(&LiveQuery) -> Github,
        now: DateTime<Utc>,
    ) -> Result<Output, CliFailure> {
        let flags = StatusView::current_flags(log, github)?;
        let before = HeartbeatRecord::read(git_dir).ok().flatten();
        let seen = before.as_ref().map(|record| record.latest().flags().clone()).unwrap_or_default();
        let fresh = Listed::new(flags.since(&seen));
        let judged = flags.awaiting_judgement();
        HeartbeatRecord::succeeding(before, Snapshot::new(now, flags, GcHook::run())).write(git_dir)?;
        Ok(match judged.raised() {
            Some(_) => Output::Flagged(fresh),
            None => Output::Listed(fresh),
        })
    }
}
