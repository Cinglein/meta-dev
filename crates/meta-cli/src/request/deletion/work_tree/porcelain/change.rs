use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::request::deletion::cargo_target::CargoTarget;
use crate::request::deletion::kept::Kept;

#[derive(Debug)]
pub enum Change {
    Tracked,
    Untracked,
    Ignored(PathBuf),
}

impl Change {
    const UNTRACKED: &'static str = "?? ";
    const IGNORED: &'static str = "!! ";

    pub fn kept(&self, root: &Path) -> Option<Kept> {
        match self {
            Self::Tracked => Some(Kept::Uncommitted),
            Self::Untracked => Some(Kept::Untracked),
            Self::Ignored(path) => CargoTarget::verified(&root.join(path)).err().map(Kept::nested),
        }
    }
}

impl FromStr for Change {
    type Err = Kept;

    fn from_str(entry: &str) -> Result<Self, Self::Err> {
        Ok(match (entry.strip_prefix(Self::UNTRACKED), entry.strip_prefix(Self::IGNORED)) {
            (Some(_), _) => Self::Untracked,
            (None, Some(path)) => Self::Ignored(PathBuf::from(path)),
            (None, None) => Self::Tracked,
        })
    }
}
