use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::request::local::local_clone::vetting::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Tracked,
    Untracked,
    Ignored(PathBuf),
}

impl Change {
    const UNTRACKED: &'static str = "?? ";
    const IGNORED: &'static str = "!! ";

    pub fn kept(&self) -> Option<Kept> {
        match self {
            Self::Tracked => Some(Kept::Uncommitted),
            Self::Untracked => Some(Kept::Untracked),
            Self::Ignored(_) => None,
        }
    }

    pub fn ignored(&self) -> Option<&Path> {
        match self {
            Self::Ignored(path) => Some(path),
            Self::Tracked | Self::Untracked => None,
        }
    }
}

impl FromStr for Change {
    type Err = std::convert::Infallible;

    fn from_str(entry: &str) -> Result<Self, Self::Err> {
        Ok(match (entry.strip_prefix(Self::UNTRACKED), entry.strip_prefix(Self::IGNORED)) {
            (Some(_), _) => Self::Untracked,
            (None, Some(path)) => Self::Ignored(PathBuf::from(path)),
            (None, None) => Self::Tracked,
        })
    }
}
