mod turn;
mod watched;

use std::time::{Duration, Instant};

use meta_log::LogStore as _;
use meta_pull_query::LiveQuery;
use meta_pulls::{Action, GhRefusal, Stop, Trunk};
use meta_schema::PullRequest;
use meta_state::State;

use super::close_out::CloseOut;
use super::gh_calls::GhCalls;
use crate::request::cli_failure::CliFailure;
use crate::request::live_github::LiveGithub;
use crate::request::output::Listed;
use crate::request::project::{CiSettings, Project};
use turn::Turn;
use watched::Watched;

#[derive(Debug)]
pub struct HeadWatch<'run> {
    project: &'run Project,
    trunk: &'run Trunk,
    pace: Duration,
    pull_request: PullRequest,
    started: Instant,
}

impl<'run> HeadWatch<'run> {
    pub fn new(project: &'run Project, trunk: &'run Trunk, pace: Duration, pull_request: PullRequest) -> Self {
        Self { project, trunk, pace, pull_request, started: Instant::now() }
    }

    pub fn pull_request(&self) -> PullRequest {
        self.pull_request
    }

    pub fn trunk(&self) -> &Trunk {
        self.trunk
    }

    pub fn watched(self) -> Result<Watched, CliFailure> {
        let turns = self.project.ci_settings().queue_turns(self.pace);
        let ended = turns.map(|()| self.polled().transpose()).find_map(std::convert::identity);
        ended.unwrap_or(Ok(Watched::Stopped(Stop::ChecksTimeout(self.pull_request))))
    }

    pub fn acted(&self, action: Action) -> Result<Option<Watched>, CliFailure> {
        let called = action.call().map(|call| GhCalls::new(self.project).called(call, action.operands()));
        called.and_then(Result::err).map_or_else(|| Ok(self.paced()), |refusal| self.rechecked(action, refusal))
    }

    pub fn closed(&self, close_out: CloseOut) -> Result<Listed, CliFailure> {
        close_out.entered(&self.project.log(self.project.ci_settings().budget(CiSettings::ROUTINE))?)
    }

    fn paced(&self) -> Option<Watched> {
        std::thread::sleep(self.pace);
        None
    }

    fn polled(&self) -> Result<Option<Watched>, CliFailure> {
        let patience = self.project.ci_settings().queue_patience(self.started.elapsed());
        self.turned()?.map_or_else(
            |refusal| Ok(Some(Watched::Stopped(Stop::Unread { pull_request: self.pull_request, refusal }))),
            |turn| turn.taken(patience),
        )
    }

    fn rechecked(&self, action: Action, refusal: GhRefusal) -> Result<Option<Watched>, CliFailure> {
        let patience = self.project.ci_settings().queue_patience(self.started.elapsed());
        let landed = self.turned()?.ok().and_then(|turn| turn.landed(patience)).transpose()?;
        let noted = landed.flatten().map(|watched| watched.noted(&refusal));
        Ok(noted.or(Some(Watched::Stopped(Stop::Refused { action, refusal }))))
    }

    fn turned(&self) -> Result<Result<Turn<'_>, GhRefusal>, CliFailure> {
        let settings = self.project.ci_settings();
        let state = State::fold(self.project.log(settings.budget(CiSettings::ROUTINE))?.read()?.records());
        let task = state.tasks().iter().find(|task| task.pull_requests().contains(&self.pull_request));
        let linked = task.map(|task| task.pull_requests().to_vec()).unwrap_or_default();
        let query = LiveQuery::new(linked.into_iter().chain([self.pull_request]).collect());
        let live = LiveGithub::for_project(self.project).live(&query);
        Ok(live.map(|live| Turn::new(self, state, live)).map_err(GhRefusal::new))
    }
}
