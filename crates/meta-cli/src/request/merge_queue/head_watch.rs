mod turn;

use std::time::Instant;

use meta_log::LogStore as _;
use meta_pull_query::LiveQuery;
use meta_pulls::{Action, Stop};
use meta_run::{GhArgv, Port, Run};
use meta_schema::PullRequest;
use meta_state::State;

use super::close_out::CloseOut;
use super::watched::Watched;
use crate::request::cli_failure::CliFailure;
use crate::request::live_github::LiveGithub;
use crate::request::output::Listed;
use crate::request::project::{CiSettings, Project};
use turn::Turn;

#[derive(Debug)]
pub struct HeadWatch<'project> {
    project: &'project Project,
    pull_request: PullRequest,
    started: Instant,
}

impl<'project> HeadWatch<'project> {
    pub fn new(project: &'project Project, pull_request: PullRequest) -> Self {
        Self { project, pull_request, started: Instant::now() }
    }

    pub fn pull_request(&self) -> PullRequest {
        self.pull_request
    }

    pub fn watched(self) -> Result<Watched, CliFailure> {
        let ended = self.project.ci_settings().queue_turns().find_map(|()| self.polled().transpose());
        ended.unwrap_or(Ok(Watched::Stopped(Stop::ChecksTimeout(self.pull_request))))
    }

    pub fn acted(&self, action: &Action) {
        let settings = self.project.ci_settings();
        let call = action.tested().map_or(GhArgv::UpdateBranch, |_| GhArgv::SquashMerge);
        let run = Port::<GhArgv>::named(settings.program(), self.project.root()).run(call, action.operands());
        let deadline = Instant::now() + settings.gh_limit();
        action.waits().map_or_else(|| self.named(run).reported(deadline).ok().map(drop), |()| None);
    }

    pub fn paced(&self) -> Option<Watched> {
        std::thread::sleep(self.project.ci_settings().queue_pace().duration());
        None
    }

    pub fn closed(&self, close_out: CloseOut) -> Result<Listed, CliFailure> {
        close_out.entered(&self.project.log(self.project.ci_settings().budget(CiSettings::ROUTINE))?)
    }

    fn polled(&self) -> Result<Option<Watched>, CliFailure> {
        let settings = self.project.ci_settings();
        let state = State::fold(self.project.log(settings.budget(CiSettings::ROUTINE))?.read()?.records());
        let task = state.tasks().iter().find(|task| task.pull_requests().contains(&self.pull_request));
        let linked = task.map(|task| task.pull_requests().to_vec()).unwrap_or_default();
        let query = LiveQuery::new(linked.into_iter().chain([self.pull_request]).collect());
        let poll = LiveGithub::for_project(self.project).live(&query).ok();
        let patience = settings.queue_patience(self.started.elapsed());
        poll.and_then(|live| Turn::new(self, state, live).taken(patience)).unwrap_or_else(|| Ok(self.paced()))
    }

    fn named(&self, run: Run) -> Run {
        self.project.repository().ok().iter().fold(run, |run, repository| repository.named(run))
    }
}
