use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::entity::{Concern, Issue, Task};
use crate::id::Id;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Entry {
    Concern(Id<Concern>),
    Issue(Id<Issue>),
    Task(Id<Task>),
}

#[derive(Debug, Error)]
pub enum EntryFailure {
    #[error("`{0}` is not a concern, issue or task id")]
    Unknown(String),
}

impl Entry {
    pub fn concern(&self) -> Option<Id<Concern>> {
        match self {
            Self::Concern(id) => Some(*id),
            Self::Issue(_) | Self::Task(_) => None,
        }
    }

    pub fn issue(&self) -> Option<Id<Issue>> {
        match self {
            Self::Issue(id) => Some(*id),
            Self::Concern(_) | Self::Task(_) => None,
        }
    }

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
            .map_err(|_| EntryFailure::Unknown(text.to_owned()))
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
