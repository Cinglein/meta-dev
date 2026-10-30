use std::fmt;

use serde::{Deserialize, Serialize};

use crate::cause::Cause;
use crate::entity::Project;
use crate::id::Id;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    Instruments(Id<Project>),
    Serves(Id<Project>),
    CausedBy {
        cause: Cause,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        project: Option<Id<Project>>,
    },
}

impl fmt::Display for Purpose {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instruments(project) => write!(formatter, "instruments {project}"),
            Self::Serves(project) => write!(formatter, "serves {project}"),
            Self::CausedBy { cause, project } => {
                write!(formatter, "caused_by {cause}")?;
                project.map_or(Ok(()), |project| write!(formatter, " in {project}"))
            }
        }
    }
}
