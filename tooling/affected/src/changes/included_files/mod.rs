mod include_scan;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use include_scan::IncludeScan;

use crate::reach::Reach;
use task::CrateName;
use task::PackageGraph;
use task::SourceFile;

#[derive(Default)]
pub struct IncludedFiles(BTreeMap<PathBuf, BTreeSet<CrateName>>);

impl IncludedFiles {
    #[must_use]
    pub fn collect(files: &[SourceFile]) -> IncludedFiles {
        IncludedFiles(
            files
                .iter()
                .filter_map(|file| {
                    file.crate_name()
                        .map(|name| (name, IncludeScan::in_file(file)))
                })
                .fold(BTreeMap::new(), |feeders, (name, included)| {
                    included.into_iter().fold(feeders, |mut feeders, path| {
                        feeders.entry(path).or_default().insert(name.clone());
                        feeders
                    })
                }),
        )
    }

    #[must_use]
    pub fn reach(&self, path: &Path, packages: &PackageGraph) -> Vec<Reach> {
        Reach::reached_by(path, packages).map_or_else(|| self.fed(path), |reach| vec![reach])
    }

    fn fed(&self, path: &Path) -> Vec<Reach> {
        self.0.get(path).map_or_else(
            || vec![Reach::Everything],
            |names| names.iter().cloned().map(Reach::Package).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{IncludedFiles, Reach};
    use std::path::Path;
    use task::{PackageGraph, SourceFile};
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "task", "manifest_path": "/w/tooling/task/Cargo.toml", "dependencies": []},
        {"name": "eval", "manifest_path": "/w/crates/eval/Cargo.toml", "dependencies": []}
    ]}"#;
    const POLICY: &str = "pub struct Policy; impl Policy { const DEFAULTS: &str = include_str!(\"../../../policy.toml\"); }";
    const NETWORK: &str = "pub const NETWORK: &[u8] = include_bytes!(\"./network.bin\"); pub const OWNER: &str = env!(\"../../../LICENSE\");";
    const FILES: [(&str, &str, &str); 2] = [
        ("task", "tooling/task/src/policy/mod.rs", POLICY),
        ("eval", "crates/eval/src/network.rs", NETWORK),
    ];
    const FED: [(&str, &str); 2] = [
        ("tooling/policy.toml", "task"),
        ("crates/eval/src/network.bin", "eval"),
    ];
    const UNFED: &str = "LICENSE";

    #[test]
    fn a_root_file_reaches_every_package_that_includes_it_and_an_unincluded_one_everything() {
        let packages = PackageGraph::from_metadata(METADATA).unwrap();
        let files = FILES.map(|(name, path, text)| {
            SourceFile::parse(path, text.to_owned())
                .unwrap()
                .in_crate(name.parse().unwrap())
        });
        let included = IncludedFiles::collect(&files);
        let reached = FED.map(|(path, _)| included.reach(Path::new(path), &packages));
        assert_eq!(
            reached,
            FED.map(|(_, name)| vec![Reach::Package(name.parse().unwrap())])
        );
        assert_eq!(
            included.reach(Path::new(UNFED), &packages),
            vec![Reach::Everything]
        );
    }
}
