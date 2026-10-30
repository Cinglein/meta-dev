mod project_progress;
mod project_state;

use std::collections::HashMap;

use meta_schema::{Acceptance, Id, NewProject, Project, Purpose, Task};

use crate::project_row::ProjectRow;
use crate::project_stage::ProjectStage;
use crate::refusal::Refusal;
pub use project_progress::ProjectProgress;
use project_state::ProjectState;

#[derive(Clone, Debug, Default)]
pub struct Projects {
    states: Vec<ProjectState>,
    purposes: HashMap<Id<Task>, Purpose>,
}

impl Projects {
    pub fn propose(&mut self, project: &NewProject) -> Result<(), Refusal> {
        self.state(project.project()).map_or(Ok(()), |_| Err(Refusal::DuplicateId))?;
        self.states.push(ProjectState::proposed(project.clone()));
        Ok(())
    }

    pub fn advance(
        &mut self,
        project: Id<Project>,
        step: impl FnOnce(&ProjectProgress) -> Result<ProjectProgress, Refusal>,
    ) -> Result<(), Refusal> {
        let found = self.states.iter_mut().find(|state| state.project().project() == project);
        found.ok_or(Refusal::UnknownProject(project))?.advance(step)
    }

    pub fn admit(&self, purpose: &Purpose) -> Result<(), Refusal> {
        purpose.project().map_or(Ok(()), |project| {
            purpose.visit(self.state(project).ok_or(Refusal::UnknownProject(project))?.progress())
        })
    }

    pub fn attach(&mut self, task: Id<Task>, purpose: Purpose) -> Result<(), Refusal> {
        self.admit(&purpose)?;
        (!self.purposes.contains_key(&task)).then_some(()).ok_or(Refusal::AlreadyPurposed)?;
        self.purposes.insert(task, purpose);
        Ok(())
    }

    pub fn purpose(&self, task: Id<Task>) -> Option<Purpose> {
        self.purposes.get(&task).copied()
    }

    pub fn stage(&self, project: Id<Project>, held: impl Fn(Id<Task>) -> bool) -> Option<ProjectStage> {
        let stage = self.state(project)?.progress().stage();
        let serving = |(task, purpose): (&Id<Task>, &Purpose)| *purpose == Purpose::Serves(project) && held(*task);
        Some(match stage {
            ProjectStage::Instrumented if self.purposes.iter().any(serving) => ProjectStage::Active,
            _ => stage,
        })
    }

    pub fn listed(&self, held: impl Fn(Id<Task>) -> bool) -> Vec<ProjectRow<'_>> {
        self.states
            .iter()
            .filter_map(|state| {
                let project = state.project();
                let root = state.progress().acceptance().map(Acceptance::root);
                Some(ProjectRow::new(project, self.stage(project.project(), &held)?, root))
            })
            .collect()
    }

    fn state(&self, project: Id<Project>) -> Option<&ProjectState> {
        self.states.iter().find(|state| state.project().project() == project)
    }
}
