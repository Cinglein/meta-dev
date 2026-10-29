use std::path::{Path, PathBuf};

use super::cause::Cause;
use super::kept::Kept;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Occupancy {
    Listed(Vec<PathBuf>),
    Unlisted(Cause),
}

impl Occupancy {
    pub fn vacant(&self, path: &Path) -> Result<(), Kept> {
        match self {
            Self::Listed(open) if open.iter().any(|name| name.starts_with(path)) => Err(Kept::InUse),
            Self::Listed(_) => Ok(()),
            Self::Unlisted(cause) => Err(Kept::Uninspected(cause.clone())),
        }
    }
}
