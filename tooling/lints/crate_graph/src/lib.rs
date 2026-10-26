mod crate_summary;
mod graph_report;
mod item_count;
mod item_graph;
mod test_files;

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use task::{Policy, Report, SourceFile};

use crate_summary::CrateSummary;
use graph_report::GraphReport;
use item_graph::ItemGraph;
use test_files::TestFiles;

#[derive(Debug)]
pub struct CrateGraph;

impl CrateGraph {
    const SOURCES: &str = "src";

    #[must_use]
    pub fn report(files: &[SourceFile], policy: &Policy) -> Report {
        Report::new(
            "a crate whose items form one big cycle or two loosely linked groups is split",
            Self::summaries(files)
                .iter()
                .flat_map(|summary| summary.violations(policy))
                .collect(),
        )
    }

    #[must_use]
    pub fn graphs(files: &[SourceFile]) -> impl fmt::Display {
        GraphReport::new(Self::summaries(files))
    }

    fn summaries(files: &[SourceFile]) -> Vec<CrateSummary> {
        let tests = TestFiles::collect(files);
        files
            .iter()
            .filter(|file| !tests.contains(file.path()))
            .filter_map(|file| Self::crate_of(file.path()).map(|root| (root, file)))
            .fold(
                BTreeMap::<PathBuf, Vec<&SourceFile>>::new(),
                |mut crates, (root, file)| {
                    crates.entry(root).or_default().push(file);
                    crates
                },
            )
            .into_iter()
            .map(|(root, files)| CrateSummary::of_graph(root, &ItemGraph::of_files(&files)))
            .collect()
    }

    fn crate_of(path: &Path) -> Option<PathBuf> {
        let components: Vec<Component> = path.components().collect();
        components
            .iter()
            .position(|component| component.as_os_str() == Self::SOURCES)
            .map(|sources| components.iter().take(sources).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::CrateGraph;
    use task::SourceFile;
    const FILES: [(&str, &str); 2] = [
        (
            "crates/one/src/lib.rs",
            "pub struct Ant(Bee); pub struct Bee(Ant, Cow); pub struct Cow;",
        ),
        ("crates/two/src/lib.rs", "pub struct Elk;"),
    ];
    const REPORT: &str = "crates/one: 3 items, 3 edges\n  cycle of 2 items: Ant Bee\n  best cut: conductance 1.000 between 2 items [Ant Bee] and 1 items [Cow]\ncrates/two: 1 items, 0 edges\n";

    #[test]
    fn the_graph_shows_each_crate_with_its_cycles_and_best_cut() {
        let files = FILES.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        assert_eq!(CrateGraph::graphs(&files).to_string(), REPORT);
    }
}
