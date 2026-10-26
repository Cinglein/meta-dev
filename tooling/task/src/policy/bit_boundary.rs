use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BitBoundary(Vec<PathBuf>);

impl BitBoundary {
    #[must_use]
    pub fn covers(&self, path: &Path) -> bool {
        self.0.iter().any(|boundary| path.starts_with(boundary))
    }
}
