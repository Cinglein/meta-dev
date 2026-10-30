mod built_target;
mod dependency;

use std::path::{Path, PathBuf};

use built_target::BuiltTarget;
pub use dependency::Dependency;
use serde::Deserialize;
use serde::de::IgnoredAny;

use crate::crate_name::CrateName;

#[derive(Debug, Deserialize)]
pub struct Package {
    name: CrateName,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
    #[serde(default)]
    targets: Vec<BuiltTarget>,
    #[serde(default)]
    publish: Option<Vec<IgnoredAny>>,
}

impl Package {
    pub fn name(&self) -> &CrateName {
        &self.name
    }

    pub fn publish(&self) -> Option<&[IgnoredAny]> {
        self.publish.as_deref()
    }

    pub fn doctested(&self) -> bool {
        self.targets
            .iter()
            .find_map(BuiltTarget::doctested)
            .is_some()
    }

    pub fn example(&self) -> Option<&BuiltTarget> {
        self.targets
            .iter()
            .find(|target| target.example().is_some())
    }

    pub fn directory(&self, workspace_root: &Path) -> PathBuf {
        self.manifest_path
            .parent()
            .map(|directory| directory.strip_prefix(workspace_root).unwrap_or(directory))
            .map(Path::to_path_buf)
            .unwrap_or_default()
    }

    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    pub fn local_dependencies(&self) -> impl Iterator<Item = &CrateName> {
        self.dependencies.iter().filter_map(Dependency::local_name)
    }
}
