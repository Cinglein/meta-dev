use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::failure::Failure;

#[derive(Clone, Debug, Deserialize)]
pub struct TargetDirectory {
    target_directory: PathBuf,
}

impl TargetDirectory {
    pub fn from_metadata(json: &str) -> Result<TargetDirectory, Failure> {
        Ok(serde_json::from_str(json)?)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.target_directory
    }
}
