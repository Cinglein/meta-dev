use std::fmt;

use serde::{Deserialize, Serialize};

use crate::level_name::LevelName;
use crate::rating::Rating;
use crate::scenario::Scenario;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "step", rename_all = "snake_case")]
pub enum Change {
    First { level: LevelName },
    Lowered { from: LevelName, level: LevelName },
    Raised { from: LevelName, level: LevelName, scenario: Scenario },
}

impl Change {
    #[must_use]
    pub fn before(&self) -> Rating {
        match self {
            Self::First { .. } => Rating::Unrated,
            Self::Lowered { from, .. } | Self::Raised { from, .. } => Rating::Rated(from.clone()),
        }
    }

    #[must_use]
    pub fn level(&self) -> &LevelName {
        match self {
            Self::First { level } | Self::Lowered { level, .. } | Self::Raised { level, .. } => level,
        }
    }
}

impl fmt::Display for Change {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::First { level } => write!(formatter, "{level}"),
            Self::Lowered { from, level } => write!(formatter, "{level} (lowered from {from})"),
            Self::Raised { from, level, scenario } => write!(formatter, "{level} (raised from {from}: {scenario})"),
        }
    }
}
