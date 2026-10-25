use std::fmt;
use std::path::PathBuf;

use crate::path_prefix::PathPrefix;

#[derive(Clone, Debug)]
pub struct ChangedPath(PathBuf);

impl ChangedPath {
    pub fn is_under(&self, prefix: &PathPrefix) -> bool {
        self.0.starts_with(prefix)
    }
}

impl From<&str> for ChangedPath {
    fn from(path: &str) -> ChangedPath {
        ChangedPath(PathBuf::from(path))
    }
}

impl fmt::Display for ChangedPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.display())
    }
}
