use std::fmt;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MergeState {
    Behind,
    Blocked,
    Clean,
    Dirty,
    Draft,
    HasHooks,
    Unstable,
    #[default]
    #[serde(other)]
    Unknown,
}

impl fmt::Display for MergeState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Behind => "behind",
            Self::Blocked => "blocked",
            Self::Clean => "clean",
            Self::Dirty => "dirty",
            Self::Draft => "draft",
            Self::HasHooks => "has-hooks",
            Self::Unstable => "unstable",
            Self::Unknown => "unknown",
        })
    }
}
