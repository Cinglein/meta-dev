use meta_log::LogStore;
use meta_schema::{Actor, Id, Task, Text};
use meta_state::State;

use super::{Brief, BriefOptions, Rules};
use crate::request::cli_failure::{CliFailure, Invalid, Setup};
use crate::request::output::{Listed, Output};
use crate::request::project::{CiSettings, Project};

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
        let held = state.tasks().iter().find(|held| held.new_task().task() == self.task);
        let task = held.ok_or_else(|| CliFailure::raised::<Invalid>(format!("unknown task {}", self.task)))?;
        let file = project.root().join(Rules::FILE);
        let failed = |reason: &dyn std::fmt::Display| CliFailure::raised::<Setup>(format!("{}: {reason}", Rules::FILE));
        let text = std::fs::read_to_string(&file).map_err(|reason| failed(&reason))?;
        let rules = text.parse::<Rules>().map_err(|reason| failed(&reason))?;
        let repository = project.repository().map_err(|reason| CliFailure::raised::<Setup>(reason.into_listed()))?;
        let actor = self.actor.map_or_else(default_actor, Ok)?;
        let options = BriefOptions::new(actor, self.scratch, repository);
        Ok(Output::Listed(Listed::new(Brief::new(task.new_task().clone(), rules, options))))
    }
}
