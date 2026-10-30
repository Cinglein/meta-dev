use std::fmt;

use meta_acceptance::{Gating, Observability};
use serde::{Deserialize, Serialize};

use crate::entity::Project;
use crate::id::Id;
use crate::root::Root;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acceptance {
    project: Id<Project>,
    root: Root,
    #[serde(flatten)]
    observability: Observability,
    enforcement: Gating,
}

impl Acceptance {
    #[must_use]
    pub fn new(project: Id<Project>, root: Root, observability: Observability, enforcement: Gating) -> Self {
        Self { project, root, observability, enforcement }
    }

    #[must_use]
    pub fn project(&self) -> Id<Project> {
        self.project
    }

    #[must_use]
    pub fn root(&self) -> Root {
        self.root
    }
}

impl fmt::Display for Acceptance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.project, self.root, self.observability)?;
        self.enforcement.points().as_slice().iter().try_for_each(|point| write!(formatter, " gate {point}"))
    }
}
