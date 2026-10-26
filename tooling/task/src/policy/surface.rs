use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Surface(PathBuf);

impl Surface {
    #[must_use]
    pub fn covers(&self, path: &Path) -> bool {
        path.starts_with(&self.0)
    }
}

impl From<&str> for Surface {
    fn from(path: &str) -> Surface {
        Surface(PathBuf::from(path))
    }
}
