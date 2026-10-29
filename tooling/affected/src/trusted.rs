use task::BranchName;
use task::PackageGraph;
use task::Selection;

use crate::Affected;
use crate::reach::Reach;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trusted(Vec<BranchName>);

impl Trusted {
    #[must_use]
    pub fn new(commits: Vec<BranchName>) -> Trusted {
        Trusted(commits)
    }

    pub fn commits(&self) -> &[BranchName] {
        &self.0
    }

    #[must_use]
    pub fn tested(graph: &PackageGraph, reaches: &[Vec<Reach>]) -> Selection {
        reaches
            .iter()
            .map(|diff| Affected::reached(graph, diff))
            .fold(Selection::Workspace, |tested, demanded| match demanded {
                Selection::Workspace => tested,
                Selection::Crates(crates) => tested.within(crates.iter()),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::Trusted;
    use crate::reach::Reach;
    use std::collections::BTreeSet;
    use std::path::Path;
    use task::{PackageGraph, Selection};
    const PACKAGES: usize = 3;
    const EDGES: [(usize, usize); 3] = [(0, 1), (0, 2), (1, 2)];
    const PATHS: [&str; 6] = [
        "crates/p0/a.rs",
        "crates/p1/a.rs",
        "crates/p2/a.rs",
        "Cargo.lock",
        "tooling/unknown.toml",
        "README.md",
    ];

    fn metadata(edges: usize) -> String {
        let package = |i: usize| {
            let deps: Vec<String> = EDGES
                .iter()
                .enumerate()
                .filter(|(bit, (_, from))| edges >> bit & 1 == 1 && *from == i)
                .map(|(_, (to, _))| format!(r#"{{"name": "p{to}", "path": "/w/crates/p{to}"}}"#))
                .collect();
            format!(r#"{{"name": "p{i}", "manifest_path": "/w/crates/p{i}/Cargo.toml", "dependencies": [{}]}}"#, deps.join(","))
        };
        let all: Vec<String> = (0..PACKAGES).map(package).collect();
        format!(r#"{{"workspace_root": "/w", "packages": [{}]}}"#, all.join(","))
    }

    fn depends(edges: usize, from: usize, to: usize) -> bool {
        from == to
            || EDGES.iter().enumerate().any(|(bit, (low, high))| {
                edges >> bit & 1 == 1 && *high == from && depends(edges, *low, to)
            })
    }

    fn in_c(edges: usize, c: usize, path: usize) -> bool {
        path >= PACKAGES && path != PATHS.len() - 1 || path < PACKAGES && depends(edges, c, path)
    }

    fn reaches(graph: &PackageGraph, diff: usize) -> Vec<Reach> {
        (0..PATHS.len())
            .filter(|path| diff >> path & 1 == 1)
            .filter_map(|path| Reach::reached_by(Path::new(PATHS[path]), graph).or(Some(Reach::Everything)))
            .collect()
    }

    #[test]
    fn a_crate_is_reused_exactly_when_some_trusted_commit_leaves_its_inputs_identical() {
        let diffs = 0..1usize << PATHS.len();
        for edges in 0..1usize << EDGES.len() {
            let graph = PackageGraph::from_metadata(&metadata(edges)).unwrap();
            let all = graph.names();
            let tested = |trusted: &[usize]| {
                let sets: Vec<Vec<Reach>> = trusted.iter().map(|d| reaches(&graph, *d)).collect();
                Trusted::tested(&graph, &sets).within(&all)
            };
            let oracle = |trusted: &[usize]| {
                let names: BTreeSet<_> = (0..PACKAGES)
                    .filter(|c| trusted.iter().all(|d| (0..PATHS.len()).any(|p| d >> p & 1 == 1 && in_c(edges, *c, p))))
                    .map(|c| format!("p{c}").parse().unwrap())
                    .collect();
                Selection::Crates(names)
            };
            assert_eq!(tested(&[]), oracle(&[]));
            for (first, second) in diffs.clone().flat_map(|a| diffs.clone().map(move |b| (a, b))) {
                assert_eq!(tested(&[first, second]), oracle(&[first, second]), "{edges} {first} {second}");
            }
        }
    }
}
