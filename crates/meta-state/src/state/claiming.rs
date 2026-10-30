use meta_schema::{Cause, Id, Project, PurposeMissing, PurposeRule, PurposeVisitor, Task};

use crate::project_stage::ProjectStage;
use crate::refusal::Refusal;
use crate::state::State;

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
}

impl PurposeVisitor for Claiming<'_> {
    type Output = Result<(), Refusal>;

    fn instruments(self, _: Id<Project>) -> Self::Output {
        Ok(())
    }

    fn serves(self, project: Id<Project>) -> Self::Output {
        match self.state.project_stage(project) {
            Some(
                stage @ (ProjectStage::Proposed
                | ProjectStage::Accepted
                | ProjectStage::Delivered
                | ProjectStage::Regressed),
            ) => Err(Refusal::ServesNotInstrumented(stage)),
            Some(ProjectStage::Instrumented | ProjectStage::Active) | None => Ok(()),
        }
    }

    fn caused_by(self, _: Cause, _: Option<Id<Project>>) -> Self::Output {
        Ok(())
    }
}
