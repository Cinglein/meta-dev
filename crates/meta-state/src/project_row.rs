use std::fmt;

use meta_schema::{NewProject, Root};

use crate::project_stage::ProjectStage;

#[derive(Clone, Debug)]
pub struct ProjectRow<'state> {
    project: &'state NewProject,
    stage: ProjectStage,
    root: Option<Root>,
}

impl<'state> ProjectRow<'state> {
    #[must_use]
    pub fn new(project: &'state NewProject, stage: ProjectStage, root: Option<Root>) -> Self {
        Self { project, stage, root }
    }
}

impl fmt::Display for ProjectRow<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.project, self.stage)?;
        self.root.map_or(Ok(()), |root| write!(formatter, " {root}"))
    }
}
