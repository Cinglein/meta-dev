mod gave_up;

use std::process::Command;

use chrono::Utc;
use meta_door::MetaStore;
use meta_log::LogStore;
use meta_pull_query::LiveQuery;
use meta_status::Github;

use super::gc_hook::GcHook;
use super::heartbeat_record::HeartbeatRecord;
use super::pulse::Pulse;
use super::snapshot::Snapshot;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::live_github::LiveGithub;
use crate::request::local::LocalClone;
use crate::request::output::{Listed, Output};
use crate::request::project::{Project, TickLimit};
use crate::request::status_view::StatusView;
use crate::request::timed_command::TimedCommand;
use gave_up::GaveUp;

#[derive(Debug)]
pub struct Tick;

impl Tick {
    pub fn answer(
        store: &MetaStore,
        log: &impl LogStore,
        github: impl FnOnce(&LiveQuery) -> Github,
        pulse: Pulse,
    ) -> Result<Output, CliFailure> {
        let flags = StatusView::current_flags(log, github)?;
        let before = HeartbeatRecord::read(store).ok().flatten();
        let seen = before.as_ref().map(|record| record.latest().flags().clone()).unwrap_or_default();
        let fresh = Listed::new(flags.since(&seen));
        let judged = flags.awaiting_judgement();
        HeartbeatRecord::succeeding(before, Snapshot::new(pulse.now(), flags, pulse.gc_hook())).write(store)?;
        Ok(match judged.raised() {
            Some(_) => Output::Flagged(fresh),
            None => Output::Listed(fresh),
        })
    }

    pub fn supervised(command: Command, limit: TickLimit) -> Result<Output, CliFailure> {
        let finished = TimedCommand::new(command, limit.deadline())
            .finished()
            .map_err(|reason| CliFailure::raised::<Setup>(GaveUp::after(limit, reason)))?;
        finished.verdict().answered(&finished)
    }

    pub fn live(project: &Project, clone: &LocalClone) -> Result<Output, CliFailure> {
        let github = |query: &LiveQuery| LiveGithub::for_project(project).read(query);
        Self::answer(clone.store(), &project.log()?, github, Pulse::new(Utc::now(), GcHook::Pending))
    }

    pub fn reported(store: &MetaStore, gc: GcHook) -> Result<(), CliFailure> {
        HeartbeatRecord::read(store)?.map_or(Ok(()), |record| record.reported(gc).write(store))
    }
}
