use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct AgentRoot(PathBuf);

impl AgentRoot {
    pub fn anchored(self, checkout: &Path) -> Self {
        let joined = checkout.join(&self.0);
        Self(std::fs::canonicalize(&joined).unwrap_or(joined))
    }

    pub fn holds(&self, path: &Path) -> Option<()> {
        path.starts_with(&self.0).then_some(())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}
