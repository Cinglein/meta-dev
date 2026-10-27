use std::fmt;

use serde::{Deserialize, Serialize};

use crate::change::Change;
use crate::level_name::LevelName;
use crate::rating::Rating;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "step", rename_all = "snake_case")]
pub enum Carry {
    Unrated,
    Kept { from: LevelName, level: LevelName },
    Changed { change: Change },
}

impl Carry {
    #[must_use]
    pub fn before(&self) -> Rating {
        match self {
            Self::Unrated => Rating::Unrated,
            Self::Kept { from, .. } => Rating::Rated(from.clone()),
            Self::Changed { change } => change.before(),
        }
    }

    #[must_use]
    pub fn level(&self) -> Option<&LevelName> {
        match self {
            Self::Unrated => None,
            Self::Kept { level, .. } => Some(level),
            Self::Changed { change } => Some(change.level()),
        }
    }
}

impl fmt::Display for Carry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unrated => Ok(()),
            Self::Kept { from, level } if from == level => write!(formatter, " !{level}"),
            Self::Kept { from, level } => write!(formatter, " !{level} (same as {from})"),
            Self::Changed { change } => write!(formatter, " !{change}"),
        }
    }
}
