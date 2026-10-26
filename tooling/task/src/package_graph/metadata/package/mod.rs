mod dependency;

use std::path::{Path, PathBuf};

use dependency::Dependency;
use serde::Deserialize;

use crate::crate_name::CrateName;

#[derive(Debug, Deserialize)]
pub struct Package {
    name: CrateName,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
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

    pub fn local_dependencies(&self) -> impl Iterator<Item = &CrateName> {
        self.dependencies.iter().filter_map(Dependency::local_name)
    }
}
