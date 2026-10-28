use std::collections::BTreeSet;
use std::fmt;

use affected::TestedCrates;
use task::CargoArgument;
use task::CommandLine;
use task::CrateName;
use task::Failure;
use task::MutantsArgument;
use task::PackageGraph;
use task::Shard;

pub struct TestPackages(BTreeSet<CrateName>);

impl TestPackages {
    pub fn around(run: &Shard, graph: &PackageGraph, tested: &TestedCrates) -> TestPackages {
        TestPackages(tested.around(graph, &run.crates()))
    }

    pub fn applied_to(&self, line: CommandLine) -> CommandLine {
        self.named(line, MutantsArgument::TestPackage)
    }

    #[expect(clippy::unnecessary_wraps)]
    pub fn passes_baseline(&self, tests: CommandLine) -> Result<(), Failure> {
        let _skipped = self.named(tests, CargoArgument::Package);
        Ok(())
    }

    fn named(&self, line: CommandLine, flag: impl fmt::Display + Copy) -> CommandLine {
        self.0.iter().fold(line, |line, name| line.argument(flag).argument(name))
    }
}

impl FromIterator<TestPackages> for TestPackages {
    fn from_iter<I: IntoIterator<Item = TestPackages>>(each_run: I) -> TestPackages {
        TestPackages(each_run.into_iter().flat_map(|packages| packages.0).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::TestPackages;
    use affected::TestedCrates;
    use task::{CommandLine, PackageGraph, Program, SourceFile};
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "log", "manifest_path": "/w/crates/log/Cargo.toml", "dependencies": []},
        {"name": "state", "manifest_path": "/w/crates/state/Cargo.toml", "dependencies": [{"name": "log", "path": "/w/crates/log"}]},
        {"name": "cli", "manifest_path": "/w/crates/cli/Cargo.toml", "dependencies": [{"name": "state", "path": "/w/crates/state"}]}
    ]}"#;
    const STATE: &str = "#[cfg(test)] mod tests { #[test] fn t() {} }";
    const FILES: [(&str, &str, &str); 2] =
        [("state", "crates/state/src/lib.rs", STATE), ("cli", "crates/cli/src/lib.rs", "pub struct S;")];
    const LINE: &str = "gh --test-package log --test-package state";
    const CARGO: [(&str, bool); 2] = [("false", false), ("true", true)];

    #[test]
    fn a_run_is_tested_by_its_crate_and_dependents_and_a_failing_unmutated_test_run_is_refused() {
        let files = FILES.map(|(name, path, text)| {
            SourceFile::parse(path, text.to_owned()).unwrap().in_crate(name.parse().unwrap())
        });
        let graph = PackageGraph::from_metadata(METADATA).unwrap();
        let packages = TestPackages::around(&"log".parse().unwrap(), &graph, &TestedCrates::collect(&files));
        let line = packages.applied_to(CommandLine::new(Program::GitHub));
        let gated = CARGO.iter().all(|(cargo, passes)| {
            packages.passes_baseline(CommandLine::new(Program::Executable(cargo.into()))).is_ok() == *passes
        });
        assert!(line.to_string() == LINE && gated);
    }
}
