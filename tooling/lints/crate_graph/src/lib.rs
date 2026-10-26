mod crate_summary;
mod item_count;
mod item_graph;
mod test_files;

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use task::{Policy, Report, SourceFile};

use crate_summary::CrateSummary;
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
