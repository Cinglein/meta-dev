mod target_failure;

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::{Concern, Issue};
use crate::entry::Entry;
use crate::id::Id;
use crate::unparsed::Unparsed;
use target_failure::TargetFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Target {
    Concern(Id<Concern>),
    Issue(Id<Issue>),
}

impl Target {
    #[must_use]
    pub fn entry(self) -> Entry {
        match self {
            Self::Concern(id) => Entry::Concern(id),
            Self::Issue(id) => Entry::Issue(id),
        }
    }
}

impl FromStr for Target {
    type Err = TargetFailure;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.parse()
            .map(Self::Concern)
            .or_else(|_| text.parse().map(Self::Issue))
            .map_err(|_| TargetFailure::Unknown(Unparsed::from(text)))
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
