mod gave_up;

use std::path::Path;

use chrono::Utc;
use meta_log::LogStore;
use meta_pull_query::LiveQuery;
use meta_run::Run;
use meta_status::Github;

use super::gc_hook::GcHook;
use super::heartbeat_record::HeartbeatRecord;
use super::pulse::Pulse;
use super::snapshot::Snapshot;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::live_github::LiveGithub;
use crate::request::local::LocalClone;
use crate::request::output::{Listed, Output};
use crate::request::project::{CiSettings, Project, TickLimit};
use crate::request::scale_policy::ScalePolicy;
use crate::request::status_view::StatusView;
use gave_up::GaveUp;

#[derive(Debug)]
pub struct Tick;

impl Tick {
    pub fn answer(
        git_dir: &Path,
        log: &impl LogStore,
        github: impl FnOnce(&LiveQuery) -> Github,
        pulse: Pulse,
    ) -> Result<Output, CliFailure> {
        let flags = StatusView::current_flags(log, pulse.ci_run_within(), github)?;
        let before = HeartbeatRecord::read(git_dir).ok().flatten();
        let seen = before.as_ref().map(|record| record.latest().flags().clone()).unwrap_or_default();
        let fresh = Listed::new(flags.since(&seen));
        let judged = flags.awaiting_judgement();
        HeartbeatRecord::succeeding(before, Snapshot::new(pulse.now(), flags, pulse.gc_hook())).write(git_dir)?;
        Ok(match judged.raised() {
            Some(_) => Output::Flagged(fresh),
            None => Output::Listed(fresh),
        })
    }

    pub fn supervised(run: Run, limit: TickLimit) -> Result<Output, CliFailure> {
        let ran = run.ran(limit.deadline());
        let ran = ran.map_err(|reason| CliFailure::raised::<Setup>(GaveUp::after(limit, Listed::new(reason))))?;
        match (ran.success(), ran.stderr().first_line()) {
            (true, _) => Ok(Output::Listed(Listed::new(ran.stdout().text()))),
            (false, None) => Ok(Output::Flagged(Listed::new(ran.stdout().text()))),
            (false, Some(complaint)) => Err(CliFailure::raised::<Setup>(complaint)),
        }
    }

    pub fn live(project: &Project, clone: &LocalClone) -> Result<Output, CliFailure> {
        let github = |query: &LiveQuery| LiveGithub::for_project(project).read(query);
        let within = ScalePolicy::read(project.root())?.liveness().ci_run_within();
        Self::answer(
            clone.git_dir().as_ref(),
            &project.log(project.ci_settings().budget(CiSettings::ROUTINE))?,
            github,
            Pulse::new(Utc::now(), GcHook::Pending).expecting_ci_run_within(within),
        )
    }

    pub fn reported(git_dir: &Path, gc: GcHook) -> Result<(), CliFailure> {
        HeartbeatRecord::read(git_dir)?.map_or(Ok(()), |record| record.reported(gc).write(git_dir))
    }
}
