use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BaselineFile(PathBuf);

impl BaselineFile {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }

    #[must_use]
    pub fn within(&self, root: &Path) -> PathBuf {
        root.join(&self.0)
    }
}

impl From<&str> for BaselineFile {
    fn from(path: &str) -> BaselineFile {
        BaselineFile(PathBuf::from(path))
    }
}
