use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourcePath(PathBuf);

impl SourcePath {
    #[must_use]
    pub fn new(path: &Path) -> SourcePath {
        SourcePath(path.to_path_buf())
    }
}
