mod metadata;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use metadata::Metadata;

use crate::crate_name::CrateName;
use crate::failure::Failure;

#[derive(Debug)]
pub struct PackageGraph {
    directories: BTreeMap<CrateName, PathBuf>,
    dependents: BTreeMap<CrateName, BTreeSet<CrateName>>,
    doctested: BTreeSet<CrateName>,
}

impl PackageGraph {
    pub fn from_metadata(json: &str) -> Result<PackageGraph, Failure> {
        let metadata: Metadata = serde_json::from_str(json)?;
        let dependents = metadata.packages().iter().fold(
            BTreeMap::new(),
            |graph: BTreeMap<CrateName, BTreeSet<CrateName>>, package| {
                package
                    .local_dependencies()
                    .fold(graph, |mut graph, dependency| {
                        graph
                            .entry(dependency.clone())
                            .or_default()
                            .insert(package.name().clone());
                        graph
                    })
            },
        );
        Ok(PackageGraph {
            doctested: metadata
                .packages()
                .iter()
                .filter(|package| package.doctested())
                .map(|package| package.name().clone())
                .collect(),
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
            dependents: metadata
                .packages()
                .iter()
                .map(|package| (package.name().clone(), BTreeSet::new()))
                .chain(dependents)
                .fold(BTreeMap::new(), |mut all, (name, direct)| {
                    all.entry(name).or_default().extend(direct);
                    all
                }),
        })
    }

    #[must_use]
    pub fn names(&self) -> BTreeSet<CrateName> {
        self.dependents.keys().cloned().collect()
    }

    #[must_use]
    pub fn doctested(&self) -> &BTreeSet<CrateName> {
        &self.doctested
    }

    #[must_use]
    pub fn directories(&self) -> &BTreeMap<CrateName, PathBuf> {
        &self.directories
    }

    #[must_use]
    pub fn containing(&self, path: &Path) -> Option<&CrateName> {
        self.directories
            .iter()
            .filter(|(_, directory)| path.starts_with(directory))
            .max_by_key(|(_, directory)| directory.components().count())
            .map(|(name, _)| name)
    }

    #[must_use]
    pub fn dependents_closure(&self, seeds: BTreeSet<CrateName>) -> BTreeSet<CrateName> {
        let grown: BTreeSet<CrateName> = seeds
            .iter()
            .flat_map(|seed| self.dependents.get(seed).into_iter().flatten().cloned())
            .chain(seeds.iter().cloned())
            .collect();
        if grown == seeds {
            seeds
        } else {
            self.dependents_closure(grown)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PackageGraph;
    use std::collections::BTreeSet;
    use std::path::Path;
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "board", "manifest_path": "/w/crates/board/Cargo.toml", "dependencies": [{"name": "serde", "path": null}], "targets": [{"doctest": true}]},
        {"name": "search", "manifest_path": "/w/crates/search/Cargo.toml", "dependencies": [{"name": "board", "path": "/w/crates/board"}]},
        {"name": "engine", "manifest_path": "/w/crates/engine/Cargo.toml", "dependencies": [{"name": "search", "path": "/w/crates/search"}]}
    ]}"#;
    const BOARD: &str = "board";
    const ENGINE: &str = "engine";
    const CRATES: usize = 3;
    const INSIDE: &str = "crates/board/src/lib.rs";
    const OUTSIDE: &str = "xtask/src/main.rs";

    #[test]
    fn the_closure_of_a_crate_is_every_crate_that_depends_on_it_transitively_and_itself() {
        let graph = PackageGraph::from_metadata(METADATA).unwrap();
        let closure = graph.dependents_closure(BTreeSet::from([BOARD.parse().unwrap()]));
        assert_eq!(
            (
                closure.len(),
                graph.names().len(),
                closure.contains(&ENGINE.parse().unwrap())
            ),
            (CRATES, CRATES, true)
        );
    }

    #[test]
    fn a_path_belongs_to_the_package_whose_manifest_directory_contains_it() {
        let graph = PackageGraph::from_metadata(METADATA).unwrap();
        assert_eq!(
            (
                graph.containing(Path::new(INSIDE)),
                graph.containing(Path::new(OUTSIDE)),
                graph
                    .doctested()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
            ),
            (BOARD.parse().ok().as_ref(), None, vec![BOARD.to_owned()])
        );
    }
}
