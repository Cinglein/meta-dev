use meta_schema::{Id, Project, Purpose};

use crate::request::cli_failure::{CliFailure, Misuse};

#[derive(Clone, Debug, Default)]
pub struct PurposeOptions {
    purposes: Vec<Purpose>,
    project: Option<Id<Project>>,
}

impl PurposeOptions {
    #[must_use]
    pub fn adding(self, purpose: Purpose) -> Self {
        Self { purposes: [self.purposes, vec![purpose]].concat(), ..self }
    }

    #[must_use]
    pub fn placed(self, project: Id<Project>) -> Self {
        Self { project: Some(project), ..self }
    }

    #[must_use]
    pub fn project(&self) -> Option<Id<Project>> {
        self.project
    }

    pub fn purpose(&self) -> Result<Option<Purpose>, CliFailure> {
        self.single()?;
        let first = self.purposes.first().copied();
        self.project.map_or(Ok(first), |project| Self::placing(first, project))
    }

    fn single(&self) -> Result<(), CliFailure> {
        self.purposes.get(1).map_or(Ok(()), |_| {
            Err(CliFailure::raised::<Misuse>("give only one of --instruments, --serves and --caused-by"))
        })
    }

    fn placing(purpose: Option<Purpose>, project: Id<Project>) -> Result<Option<Purpose>, CliFailure> {
        purpose
            .and_then(|purpose| purpose.in_project(project))
            .map(Some)
            .ok_or_else(|| CliFailure::raised::<Misuse>("--project goes with --caused-by"))
    }
}
