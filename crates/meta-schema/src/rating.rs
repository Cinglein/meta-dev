use std::fmt;

use crate::level_name::LevelName;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Rating {
    Unrated,
    Rated(LevelName),
}

impl Rating {
    #[must_use]
    pub fn level(&self) -> Option<&LevelName> {
        match self {
            Self::Unrated => None,
            Self::Rated(level) => Some(level),
        }
    }
}

impl From<Option<LevelName>> for Rating {
    fn from(level: Option<LevelName>) -> Rating {
        level.map_or(Rating::Unrated, Rating::Rated)
    }
}

impl fmt::Display for Rating {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unrated => formatter.write_str("unrated"),
            Self::Rated(level) => level.fmt(formatter),
        }
    }
}
