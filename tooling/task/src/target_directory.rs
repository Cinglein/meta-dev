use std::path::PathBuf;

use serde::Deserialize;

use crate::failure::Failure;

#[derive(Debug, Deserialize)]
pub struct TargetDirectory {
    target_directory: PathBuf,
}

impl TargetDirectory {
    pub fn from_metadata(json: &str) -> Result<PathBuf, Failure> {
        Ok(serde_json::from_str::<TargetDirectory>(json)?.target_directory)
    }
}
