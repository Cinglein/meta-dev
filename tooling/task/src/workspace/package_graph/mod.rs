mod metadata;

use std::collections::BTreeMap;
use std::path::PathBuf;

use metadata::Metadata;

use crate::crate_name::CrateName;
use crate::failure::Failure;

#[derive(Debug)]
pub struct PackageGraph {
    directories: BTreeMap<CrateName, PathBuf>,
}

impl PackageGraph {
    pub fn from_metadata(json: &str) -> Result<PackageGraph, Failure> {
        let metadata: Metadata = serde_json::from_str(json)?;
        Ok(PackageGraph {
            directories: metadata
                .packages()
                .iter()
                .map(|package| {
                    (
                        package.name().clone(),
                        package.directory(metadata.workspace_root()),
                    )
                })
                .collect(),
        })
    }

    #[must_use]
    pub fn directories(&self) -> &BTreeMap<CrateName, PathBuf> {
        &self.directories
    }
}

#[cfg(test)]
mod tests {
    use super::PackageGraph;
    use std::path::{Path, PathBuf};
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "log", "manifest_path": "/w/crates/log/Cargo.toml", "dependencies": []},
        {"name": "xtask", "manifest_path": "/w/xtask/Cargo.toml", "dependencies": []}
    ]}"#;
    const LOG: &str = "log";
    const DIRECTORY: &str = "crates/log";

    #[test]
    fn a_package_lives_in_its_manifest_directory_relative_to_the_workspace_root() {
        let graph = PackageGraph::from_metadata(METADATA).unwrap();
        let log = LOG.parse().unwrap();
        assert_eq!(
            graph.directories().get(&log).map(PathBuf::as_path),
            Some(Path::new(DIRECTORY))
        );
    }
}
