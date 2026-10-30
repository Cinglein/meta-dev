#[cfg(test)]
pub mod tests {
    use std::collections::BTreeSet;

    use crate::Affected;
    use crate::changes::Changes;
    use crate::reach::Reach;
    use task::{CrateName, PackageGraph, Selection, SourceFile};

    pub struct Oracle {
        edges: usize,
    }

    impl Oracle {
        const PACKAGES: usize = 3;
        const EDGES: [(usize, usize); 3] = [(0, 1), (0, 2), (1, 2)];
        const PATHS: [&str; 13] = [
            "crates/p0/src/lib.rs", "crates/p1/src/deep/er/mod.rs", "crates/p2/Cargo.toml",
            "crates/p0/README.md", "crates/p00/a.rs", "crates/p1.rs", "README.md", "Cargo.lock",
            ".github/workflows/ci.yml", ".githubx/ci.yml", ".claude/settings.json",
            "shared/data.txt", "tooling/unknown.toml",
        ];
        const INCLUDER: (usize, &str, &str) = (
            1,
            "crates/p1/src/lib.rs",
            "pub const D: &str = include_str!(\"../../../shared/data.txt\");",
        );
        const SHARED: &str = "shared/data.txt";
        const IGNORED: [&str; 2] = [".claude/", ".gitignore"];
        const TRIPLES_OVER: usize = 14;

        pub fn every() -> impl Iterator<Item = Oracle> {
            (0..1usize << Self::EDGES.len()).map(|edges| Oracle { edges })
        }

        pub fn agrees(&self) -> bool {
            let graph = PackageGraph::from_metadata(&self.metadata()).unwrap();
            let diffs = Self::diffs();
            let (owner, file, text) = Self::INCLUDER;
            let source = || {
                let name = format!("p{owner}").parse().unwrap();
                Ok(vec![
                    SourceFile::parse(file, text.to_owned())
                        .unwrap()
                        .in_crate(name),
                ])
            };
            let reaches: Vec<Vec<Reach>> = diffs
                .iter()
                .map(|diff| {
                    Changes::reached(&graph, &Changes::listed(&diff.join("\n")), source).unwrap()
                })
                .collect();
            let agrees = |picked: &[usize]| {
                let sets: Vec<Vec<Reach>> = picked.iter().map(|i| reaches[*i].clone()).collect();
                let trusted: Vec<&Vec<&str>> = picked.iter().map(|i| &diffs[*i]).collect();
                Affected::tested(&graph, &sets).within(&graph.names()) == self.expected(&trusted)
            };
            let all = 0..diffs.len();
            let pairs = all
                .clone()
                .flat_map(|a| all.clone().map(move |b| vec![a, b]));
            let small: Vec<usize> = (0..Self::TRIPLES_OVER).collect();
            let small = &small;
            let triples = small.iter().flat_map(|a| {
                small
                    .iter()
                    .flat_map(move |b| small.iter().map(move |c| vec![*a, *b, *c]))
            });
            let singles = all.clone().map(|a| vec![a]);
            std::iter::once(vec![])
                .chain(singles)
                .chain(pairs)
                .chain(triples)
                .all(|picked| agrees(&picked))
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

        pub fn diffs() -> Vec<Vec<&'static str>> {
            let singles = Self::PATHS.iter().map(|path| vec![*path]);
            let pairs = Self::PATHS.iter().enumerate().flat_map(|(at, first)| {
                Self::PATHS[at + 1..]
                    .iter()
                    .map(move |second| vec![*first, *second])
            });
            std::iter::once(vec![])
                .chain(singles)
                .chain(pairs)
                .collect()
        }

        fn expected(&self, trusted: &[&Vec<&str>]) -> Selection {
            Selection::Crates(
                (0..Self::PACKAGES)
                    .filter(|c| trusted.iter().all(|diff| self.touched(*c, diff)))
                    .map(|c| format!("p{c}").parse::<CrateName>().unwrap())
                    .collect::<BTreeSet<CrateName>>(),
            )
        }

        fn touched(&self, c: usize, diff: &[&str]) -> bool {
            diff.iter().any(|path| self.hits(c, path))
        }

        fn hits(&self, c: usize, path: &str) -> bool {
            let (owner, _, _) = Self::INCLUDER;
            if path.rsplit('.').next() == Some("md")
                || Self::IGNORED
                    .iter()
                    .any(|ignored| path.starts_with(ignored))
            {
                false
            } else if let Some(package) =
                (0..Self::PACKAGES).find(|k| path.starts_with(&format!("crates/p{k}/")))
            {
                self.depends(c, package)
            } else if path == Self::SHARED {
                self.depends(c, owner)
            } else {
                true
            }
        }

        fn depends(&self, from: usize, to: usize) -> bool {
            from == to
                || Self::EDGES.iter().enumerate().any(|(bit, (low, high))| {
                    self.edges >> bit & 1 == 1 && *high == from && self.depends(*low, to)
                })
        }
    }
}
