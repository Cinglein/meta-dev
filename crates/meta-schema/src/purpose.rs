use std::fmt;

use serde::{Deserialize, Serialize};

use crate::cause::Cause;
use crate::entity::Project;
use crate::id::Id;
use crate::purpose_visitor::PurposeVisitor;

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

impl Purpose {
    #[must_use]
    pub fn project(&self) -> Option<Id<Project>> {
        match self {
            Self::Instruments(project) | Self::Serves(project) => Some(*project),
            Self::CausedBy { project, .. } => *project,
        }
    }

    #[must_use]
    pub fn cause(&self) -> Option<Cause> {
        match self {
            Self::CausedBy { cause, .. } => Some(*cause),
            Self::Instruments(_) | Self::Serves(_) => None,
        }
    }

    #[must_use]
    pub fn in_project(self, project: Id<Project>) -> Option<Self> {
        match self {
            Self::CausedBy { cause, .. } => Some(Self::CausedBy { cause, project: Some(project) }),
            Self::Instruments(_) | Self::Serves(_) => None,
        }
    }

    pub fn visit<Visitor: PurposeVisitor>(&self, visitor: Visitor) -> Visitor::Output {
        match self {
            Self::Instruments(project) => visitor.instruments(*project),
            Self::Serves(project) => visitor.serves(*project),
            Self::CausedBy { cause, project } => visitor.caused_by(*cause, *project),
        }
    }
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
