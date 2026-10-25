use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(try_from = "PathBuf")]
pub struct PathPrefix(PathBuf);

#[derive(Debug, Error)]
#[error("{0}: a path prefix is relative to the workspace root")]
pub struct AbsolutePrefix(String);

impl PathPrefix {
    pub fn covers(&self, path: &str) -> bool {
        Path::new(path).starts_with(&self.0)
    }
}

impl AsRef<Path> for PathPrefix {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl TryFrom<PathBuf> for PathPrefix {
    type Error = AbsolutePrefix;

    fn try_from(path: PathBuf) -> Result<PathPrefix, AbsolutePrefix> {
        if path.is_absolute() { Err(AbsolutePrefix(path.display().to_string())) } else { Ok(PathPrefix(path)) }
    }
}

impl fmt::Display for PathPrefix {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.display())
    }
}
