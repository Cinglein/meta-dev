use meta_log::LogStore;
use meta_schema::{Actor, Id, Task, Text};
use meta_state::State;

use super::{Brief, BriefFailure, BriefOptions, Rules};
use crate::request::cli_failure::{CliFailure, Invalid, Setup};
use crate::request::output::{Listed, Output};
use crate::request::project::{CiSettings, Project};
use crate::request::repository::Repository;

#[derive(Debug)]
pub struct BriefRequest {
    task: Id<Task>,
    actor: Option<Actor>,
    scratch: Option<Text>,
}

impl BriefRequest {
    #[must_use]
    pub fn new(task: Id<Task>, actor: Option<Actor>, scratch: Option<Text>) -> Self {
        Self { task, actor, scratch }
    }

    pub fn answer(
        self,
        project: &Project,
        default_actor: impl FnOnce() -> Result<Actor, CliFailure>,
    ) -> Result<Output, CliFailure> {
        let log = project.log(project.ci_settings().budget(CiSettings::ROUTINE))?;
        let state = State::fold(log.read()?.records());
        let rules = std::fs::read_to_string(project.root().join(Rules::FILE));
        let repository = project.repository().map_err(|reason| CliFailure::raised::<Setup>(reason.into_listed()))?;
        self.briefed(&state, rules, default_actor, repository).map(|brief| Output::Listed(Listed::new(brief)))
    }

    pub fn briefed(
        mut self,
        state: &State,
        rules: std::io::Result<String>,
        default_actor: impl FnOnce() -> Result<Actor, CliFailure>,
        repository: Repository,
    ) -> Result<Brief, CliFailure> {
        let held = state.tasks().iter().find(|held| held.new_task().task() == self.task);
        let task = held.ok_or_else(|| CliFailure::raised::<Invalid>(BriefFailure::UnknownTask(self.task)))?;
        let closed = state.status_of(task);
        closed
            .closed()
            .map_or(Ok(()), |status| Err(BriefFailure::Closed(self.task, status.clone())))
            .map_err(CliFailure::raised::<Invalid>)?;
        let text = rules.map_err(|reason| CliFailure::raised::<Setup>(BriefFailure::Unreadable(reason)))?;
        let rules = text.parse::<Rules>().map_err(CliFailure::raised::<Setup>)?;
        let actor = self.actor.take().map_or_else(default_actor, Ok)?;
        Ok(Brief::new(task.new_task().clone(), rules, BriefOptions::new(actor, self.scratch, repository)))
    }
}
