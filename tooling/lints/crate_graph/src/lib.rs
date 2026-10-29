mod crate_summary;
mod effect;
mod graph_report;
mod item_count;
mod item_graph;
mod test_files;

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use task::SourceFile;

use crate_summary::CrateSummary;
pub use graph_report::GraphReport;
use item_graph::ItemGraph;
use test_files::TestFiles;

#[derive(Debug)]
pub struct CrateGraph;

impl CrateGraph {
    const SOURCES: &str = "src";

    #[must_use]
    pub fn graphs(files: &[SourceFile]) -> GraphReport {
        let tests = TestFiles::collect(files);
        let summaries = files
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
            .collect();
        GraphReport::new(summaries)
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
        let graphs = CrateGraph::graphs(&files);
        assert_eq!(graphs.to_string(), REPORT);
        assert_eq!(graphs.largest_tangle().to_string(), LARGEST);
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
            pub struct Timer; impl Timer { pub fn go() { Utc::now(); None.unwrap_or_else(std::time::Instant::now); } }
            pub fn parse() { toml::from_str(TEXT); serde_json::to_string(TEXT); }
            pub fn quit() { std::process::exit(CODE); }
            pub struct Unix(std::os::unix::process::ExitStatusExt); pub struct Pure;",
        ),
    ];
    const WIDEST: [&str; 2] = ["72", "2"];
    const EFFECTS: &str = "  effects";
    const EFFECT_LINES: [&str; 2] = [
        "  effects: 1 of 2 units, share 50%: files 1",
        "  effects: 5 of 7 units, share 72%: process 2, environment 1, clock 1, parsing 1",
    ];
    const MENTIONS_AND_CALLS: [(&str, &str); 3] = [
        (
            "crates/exit/src/lib.rs",
            "use std::process::ExitStatus; pub struct MutantsExit(i32);
            impl MutantsExit { pub fn accepts(status: ExitStatus) -> bool { status.code().is_some() } }",
        ),
        (
            "crates/plan/src/lib.rs",
            "use std::fmt; pub struct ShardPlan(Vec<Shard>);
            impl fmt::Display for ShardPlan { fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&serde_json::to_string(&self.0).map_err(|_| fmt::Error)?) } }",
        ),
        (
            "crates/settings/src/lib.rs",
            "use std::str::FromStr; pub struct MutantsSettings;
            impl FromStr for MutantsSettings { type Err = Failure; fn from_str(settings: &str) -> Result<MutantsSettings, Failure> {
                let table = settings.parse::<toml::Table>()?; Ok(MutantsSettings) } }",
        ),
    ];
    const ONLY_CALLS: &str = "crates/exit: 1 items, 0 edges\ncrates/plan: 1 items, 0 edges\ncrates/settings: 1 items, 0 edges\n  effects: 1 of 1 units, share 100%: parsing 1\n";

    #[test]
    fn a_unit_performs_an_effect_by_calling_a_listed_function_not_by_naming_a_type() {
        let files = MENTIONS_AND_CALLS
            .map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        assert_eq!(CrateGraph::graphs(&files).to_string(), ONLY_CALLS);
    }

    #[test]
    fn effects_count_the_raw_share_of_units_per_crate_and_the_widest_family() {
        let files =
            EFFECT_FILES.map(|(path, text)| SourceFile::parse(path, text.to_owned()).unwrap());
        let graphs = CrateGraph::graphs(&files);
        let widest = [
            graphs.effect_share().to_string(),
            graphs.effect_dispersion().to_string(),
        ];
        assert_eq!(widest, WIDEST);
        let shown = graphs.to_string();
        let lines: Vec<&str> = shown
            .lines()
            .filter(|line| line.starts_with(EFFECTS))
            .collect();
        assert_eq!(lines, EFFECT_LINES);
    }
}
