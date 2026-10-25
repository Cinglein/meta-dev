mod conductance;
mod crate_summary;
mod graph_limits;
mod graph_report;
mod item_count;
mod item_graph;

pub use graph_limits::GraphLimits;
pub use graph_report::GraphReport;

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use crate::rule::report::Report;
use crate::rule::test_files::TestFiles;
use crate::source_file::SourceFile;
use crate_summary::CrateSummary;
use item_graph::ItemGraph;

pub struct CrateGraph;

impl CrateGraph {
    const SOURCES: &str = "src";

    pub fn report(files: &[SourceFile], limits: Option<GraphLimits>) -> Report {
        Report::new(
            "a crate whose items form one big cycle or two loosely linked groups is split",
            limits
                .into_iter()
                .flat_map(|limits| {
                    Self::summaries(files)
                        .into_iter()
                        .flat_map(move |(root, summary)| limits.violations(&root, &summary))
                })
                .collect(),
        )
    }

    pub fn graph_report(files: &[SourceFile]) -> GraphReport {
        GraphReport::new(Self::summaries(files).into_values().collect())
    }

    fn summaries(files: &[SourceFile]) -> BTreeMap<PathBuf, CrateSummary> {
        let tests = TestFiles::collect(files);
        files
            .iter()
            .filter(|file| !tests.contains(file.path()))
            .filter_map(|file| Self::crate_of(Path::new(file.path())).map(|root| (root, file)))
            .fold(BTreeMap::<PathBuf, Vec<&SourceFile>>::new(), |mut crates, (root, file)| {
                crates.entry(root).or_default().push(file);
                crates
            })
            .into_iter()
            .map(|(root, files)| {
                let summary = CrateSummary::of_graph(root.clone(), &ItemGraph::of_files(&files));
                (root, summary)
            })
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
