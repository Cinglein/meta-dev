use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::crate_name::CrateName;

#[derive(Debug, Deserialize)]
pub struct Package {
    name: CrateName,
    manifest_path: PathBuf,
}

impl Package {
    pub fn name(&self) -> &CrateName {
        &self.name
    }

    pub fn directory(&self, workspace_root: &Path) -> PathBuf {
        self.manifest_path
            .parent()
            .map(|directory| directory.strip_prefix(workspace_root).unwrap_or(directory))
            .map(Path::to_path_buf)
            .unwrap_or_default()
    }
}
