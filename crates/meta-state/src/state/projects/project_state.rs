use std::fmt;

use meta_schema::NewProject;

use super::project_progress::ProjectProgress;
use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectState {
    project: NewProject,
    progress: ProjectProgress,
}

impl ProjectState {
    pub fn proposed(project: NewProject) -> Self {
        Self { project, progress: ProjectProgress::Proposed }
    }

    pub fn project(&self) -> &NewProject {
        &self.project
    }

    pub fn progress(&self) -> &ProjectProgress {
        &self.progress
    }

    pub fn advance(
        &mut self,
        step: impl FnOnce(&ProjectProgress) -> Result<ProjectProgress, Refusal>,
    ) -> Result<(), Refusal> {
        self.progress = step(&self.progress)?;
        Ok(())
    }
}

impl fmt::Display for ProjectState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.project, self.progress.stage())
    }
}

#[cfg(test)]
mod tests;
