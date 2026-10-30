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

impl fmt::Display for Acceptance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.project, self.root, self.observability)?;
        self.enforcement.points().as_slice().iter().try_for_each(|point| write!(formatter, " gate {point}"))
    }
}
