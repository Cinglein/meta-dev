#[cfg(test)]
pub mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use crate::Affected;
    use crate::reach::Reach;
    use task::{CrateName, PackageGraph, Selection};

    pub struct Oracle {
        edges: usize,
    }

    impl Oracle {
        const PACKAGES: usize = 3;
        const EDGES: [(usize, usize); 3] = [(0, 1), (0, 2), (1, 2)];
        const PATHS: [&str; 6] = [
            "crates/p0/a.rs", "crates/p1/a.rs", "crates/p2/a.rs", "Cargo.lock",
            "tooling/unknown.toml", "README.md",
        ];
        const ALWAYS_IN: [usize; 2] = [3, 4];

        pub fn every() -> impl Iterator<Item = Oracle> {
            (0..1usize << Self::EDGES.len()).map(|edges| Oracle { edges })
        }

        pub fn agrees(&self) -> bool {
            let graph = PackageGraph::from_metadata(&self.metadata()).unwrap();
            let agrees = |trusted: &[BTreeSet<usize>]| {
                let reaches: Vec<Vec<Reach>> = trusted
                    .iter()
                    .map(|diff| Self::reaches(&graph, diff))
                    .collect();
                Affected::tested(&graph, &reaches).within(&graph.names()) == self.reused(trusted)
            };
            let diffs = 0..1usize << Self::PATHS.len();
            let pairs = diffs
                .clone()
                .flat_map(|a| diffs.clone().map(move |b| [Self::diff(a), Self::diff(b)]));
            agrees(&[]) && pairs.into_iter().all(|pair| agrees(&pair))
        }

        pub fn reaches(graph: &PackageGraph, diff: &BTreeSet<usize>) -> Vec<Reach> {
            diff.iter()
                .map(|path| {
                    Reach::reached_by(Path::new(Self::PATHS[*path]), graph)
                        .unwrap_or(Reach::Everything)
                })
                .collect()
        }

        pub fn metadata(&self) -> String {
            let package = |i: usize| {
                let deps: Vec<String> = Self::EDGES
                    .iter()
                    .enumerate()
                    .filter(|(bit, (_, high))| self.edges >> bit & 1 == 1 && *high == i)
                    .map(|(_, (low, _))| {
                        format!(r#"{{"name": "p{low}", "path": "/w/crates/p{low}"}}"#)
                    })
                    .collect();
                let manifest = format!("/w/crates/p{i}/Cargo.toml");
                format!(
                    r#"{{"name": "p{i}", "manifest_path": "{manifest}", "dependencies": [{}]}}"#,
                    deps.join(",")
                )
            };
            let all: Vec<String> = (0..Self::PACKAGES).map(package).collect();
            format!(
                r#"{{"workspace_root": "/w", "packages": [{}]}}"#,
                all.join(",")
            )
        }

        fn diff(mask: usize) -> BTreeSet<usize> {
            (0..Self::PATHS.len())
                .filter(|path| mask >> path & 1 == 1)
                .collect()
        }

        fn reused(&self, trusted: &[BTreeSet<usize>]) -> Selection {
            Selection::Crates(
                (0..Self::PACKAGES)
                    .filter(|c| {
                        trusted
                            .iter()
                            .all(|diff| !self.inputs(*c).is_disjoint(diff))
                    })
                    .map(|c| format!("p{c}").parse::<CrateName>().unwrap())
                    .collect(),
            )
        }

        fn inputs(&self, c: usize) -> BTreeSet<usize> {
            (0..Self::PACKAGES)
                .filter(|package| self.depends(c, *package))
                .chain(Self::ALWAYS_IN)
                .collect()
        }

        fn depends(&self, from: usize, to: usize) -> bool {
            from == to
                || Self::EDGES.iter().enumerate().any(|(bit, (low, high))| {
                    self.edges >> bit & 1 == 1 && *high == from && self.depends(*low, to)
                })
        }
    }
}
