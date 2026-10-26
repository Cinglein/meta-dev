use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::entity::{Concern, Issue};
use crate::entry::Entry;
use crate::id::Id;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Target {
    Concern(Id<Concern>),
    Issue(Id<Issue>),
}

#[derive(Debug, Error)]
pub enum TargetFailure {
    #[error("`{0}` is not a concern or issue id")]
    Unknown(String),
}

impl Target {
    #[must_use]
    pub fn concern(&self) -> Option<Id<Concern>> {
        match self {
            Self::Concern(id) => Some(*id),
            Self::Issue(_) => None,
        }
    }

    #[must_use]
    pub fn entry(self) -> Entry {
        match self {
            Self::Concern(id) => Entry::Concern(id),
            Self::Issue(id) => Entry::Issue(id),
        }
    }

    #[must_use]
    pub fn issue(&self) -> Option<Id<Issue>> {
        match self {
            Self::Issue(id) => Some(*id),
            Self::Concern(_) => None,
        }
    }
}

impl FromStr for Target {
    type Err = TargetFailure;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.parse()
            .map(Self::Concern)
            .or_else(|_| text.parse().map(Self::Issue))
            .map_err(|_| TargetFailure::Unknown(text.to_owned()))
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Concern(id) => id.fmt(formatter),
            Self::Issue(id) => id.fmt(formatter),
        }
    }
}
