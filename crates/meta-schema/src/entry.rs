mod entry_failure;

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::{Concern, Issue, Task};
use crate::id::Id;
use crate::target::Target;
use crate::unparsed::Unparsed;
use entry_failure::EntryFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Entry {
    Concern(Id<Concern>),
    Issue(Id<Issue>),
    Task(Id<Task>),
}

impl Entry {
    #[must_use]
    pub fn target(self) -> Option<Target> {
        match self {
            Self::Concern(id) => Some(Target::Concern(id)),
            Self::Issue(id) => Some(Target::Issue(id)),
            Self::Task(_) => None,
        }
    }

    #[must_use]
    pub fn task(&self) -> Option<Id<Task>> {
        match self {
            Self::Task(id) => Some(*id),
            Self::Concern(_) | Self::Issue(_) => None,
        }
    }
}

impl FromStr for Entry {
    type Err = EntryFailure;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.parse()
            .map(Self::Task)
            .or_else(|_| text.parse().map(Self::Concern))
            .or_else(|_| text.parse().map(Self::Issue))
            .map_err(|_| EntryFailure::Unknown(Unparsed::from(text)))
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Concern(id) => id.fmt(formatter),
            Self::Issue(id) => id.fmt(formatter),
            Self::Task(id) => id.fmt(formatter),
        }
    }
}
