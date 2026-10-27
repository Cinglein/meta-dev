mod crate_summary;
mod effect;
mod graph_report;
mod item_count;
mod item_graph;
mod test_files;

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use task::{Amount, Most, Percent, Policy, Report, SourceFile};

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
    pub fn largest_tangle(files: &[SourceFile]) -> Amount {
        let largest = Self::summaries(files)
            .iter()
            .map(CrateSummary::largest_tangle)
            .max()
            .unwrap_or_default();
        Amount::Count(Most::from(largest.count()))
    }

    #[must_use]
    pub fn effect_share(files: &[SourceFile]) -> Percent {
        Self::summaries(files)
            .iter()
            .map(|summary| summary.effects().share())
            .max()
            .unwrap_or(Percent::NONE)
    }

    #[must_use]
    pub fn effect_dispersion(files: &[SourceFile]) -> Most {
        let widest = Self::summaries(files)
            .iter()
            .map(|summary| summary.effects().dispersion())
            .max()
            .unwrap_or_default();
        Most::from(widest.count())
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
    const LARGEST: &str = "2";
    const REPORT: &str = "crates/one: 3 items, 3 edges\n  cycle of 2 items: Ant Bee\n  best cut: conductance 1.000 between 2 items [Ant Bee] and 1 items [Cow]\ncrates/two: 1 items, 0 edges\n";

    #[test]
    fn the_graph_shows_each_crate_with_its_cycles_and_best_cut() {
        let files = FILES.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        assert_eq!(CrateGraph::graphs(&files).to_string(), REPORT);
        assert_eq!(CrateGraph::largest_tangle(&files).to_string(), LARGEST);
    }

    const EFFECT_FILES: [(&str, &str); 3] = [
        (
            "crates/core/src/lib.rs",
            "use std::time::Duration; pub struct Board(toml::Value, Duration); pub fn rank() { Duration::ZERO; chrono::Utc::today(); }",
        ),
        (
            "crates/adapter/src/lib.rs",
            "use std::fs::{self, File}; pub struct Store; impl Store { pub fn load() { fs::read(PATH); } } pub struct Plain;",
        ),
        (
            "crates/dispersed/src/lib.rs",
            "use std::process::Command as Run; use std::env; use chrono::Utc;
            pub struct Spawner; impl Spawner { pub fn go() { Run::new(NAME); } }
            pub struct Reader; impl Reader { pub fn go() { env::var(NAME); } }
            pub struct Timer; impl Timer { pub fn go() { Utc::now(); std::time::Instant::now(); } }
            pub fn parse() { toml::from_str(TEXT); serde_json::to_string(TEXT); }
            pub fn quit() { std::process::exit(CODE); }
            pub struct Unix(std::os::unix::process::ExitStatusExt); pub struct Pure;",
        ),
    ];
    const WIDEST: [&str; 2] = ["72", "3"];
    const EFFECTS: &str = "  effects";
    const EFFECT_LINES: [&str; 2] = [
        "  effects: 1 of 2 units, share 0%: files 1",
        "  effects: 6 of 7 units, share 72%: process 3, environment 1, clock 1, parsing 1",
    ];

    #[test]
    fn effects_count_units_per_crate_past_one_adapter_and_the_widest_family() {
        let files =
            EFFECT_FILES.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        let widest = [
            CrateGraph::effect_share(&files).to_string(),
            CrateGraph::effect_dispersion(&files).to_string(),
        ];
        assert_eq!(widest, WIDEST);
        let shown = CrateGraph::graphs(&files).to_string();
        let lines: Vec<&str> = shown
            .lines()
            .filter(|line| line.starts_with(EFFECTS))
            .collect();
        assert_eq!(lines, EFFECT_LINES);
    }
}
