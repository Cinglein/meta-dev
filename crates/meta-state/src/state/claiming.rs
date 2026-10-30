use meta_schema::{Cause, Entry, Id, Project, Purpose, PurposeMissing, PurposeRule, PurposeVisitor, Task};

use crate::project_stage::ProjectStage;
use crate::refusal::Refusal;
use crate::state::State;
use crate::unservable::Unservable;

#[derive(Debug)]
pub struct Claiming<'state> {
    state: &'state State,
    task: Id<Task>,
}

impl<'state> Claiming<'state> {
    pub fn new(state: &'state State, task: Id<Task>) -> Self {
        Self { state, task }
    }

    pub fn allowed(self) -> Result<(), Refusal> {
        match self.state.projects.purpose(self.task) {
            Some(purpose) => purpose.visit(self),
            None => PurposeRule::under(self.state.schema())
                .admitting(None)
                .map_err(|PurposeMissing| Refusal::Unlinked(self.task)),
        }
    }

    fn open_instruments(&self, project: Id<Project>) -> Vec<Id<Task>> {
        self.state
            .tasks()
            .iter()
            .map(|task| task.new_task().task())
            .filter(|task| self.state.projects.purpose(*task) == Some(Purpose::Instruments(project)))
            .filter(|task| self.state.closed(Entry::Task(*task)).is_none())
            .collect()
    }
}

impl PurposeVisitor for Claiming<'_> {
    type Output = Result<(), Refusal>;

    fn instruments(self, _: Id<Project>) -> Self::Output {
        Ok(())
    }

    fn serves(self, project: Id<Project>) -> Self::Output {
        let unservable = |reason: Unservable| Err(Refusal::Unservable(reason));
        match self.state.project_stage(project).ok_or(Refusal::UnknownProject(project))? {
            ProjectStage::Instrumented | ProjectStage::Active => Ok(()),
            ProjectStage::Accepted => unservable(Unservable::accepted(project, self.open_instruments(project))),
            ProjectStage::Delivered => unservable(Unservable::Delivered(project)),
            ProjectStage::Regressed => unservable(Unservable::Regressed(project)),
            ProjectStage::Proposed => Err(Refusal::ProjectIs(ProjectStage::Proposed)),
        }
    }

    fn caused_by(self, _: Cause, _: Option<Id<Project>>) -> Self::Output {
        Ok(())
    }
}
