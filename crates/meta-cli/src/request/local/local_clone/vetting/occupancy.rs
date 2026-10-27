use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::cause::Cause;
use super::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Occupancy {
    Listed(Vec<PathBuf>),
    Unlisted(Cause),
}

impl Occupancy {
    const NAMED: char = 'n';

    pub fn vacant(&self, path: &Path) -> Result<(), Kept> {
        match self {
            Self::Listed(open) if open.iter().any(|name| name.starts_with(path)) => Err(Kept::InUse),
            Self::Listed(_) => Ok(()),
            Self::Unlisted(cause) => Err(Kept::Uninspected(cause.clone())),
        }
    }
}

impl FromStr for Occupancy {
    type Err = std::convert::Infallible;

    fn from_str(listed: &str) -> Result<Self, Self::Err> {
        Ok(Self::Listed(listed.lines().filter_map(|line| line.strip_prefix(Self::NAMED)).map(PathBuf::from).collect()))
    }
}
