mod package;

use std::path::{Path, PathBuf};

use package::Package;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    packages: Vec<Package>,
    workspace_root: PathBuf,
}

impl Metadata {
    pub fn packages(&self) -> &[Package] {
        &self.packages
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }
}
