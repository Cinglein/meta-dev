use std::collections::BTreeSet;
use std::fmt;

use super::MutantsBuild;
use affected::TestedCrates;
use task::CargoArgument;
use task::CommandLine;
use task::CrateName;
use task::MutantsArgument;
use task::PackageGraph;
use task::Shard;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct TestPackages(BTreeSet<CrateName>);

impl TestPackages {
    pub fn around(run: &Shard, graph: &PackageGraph, tested: &TestedCrates) -> TestPackages {
        TestPackages(tested.around(graph, &run.crates()))
    }

    pub fn each_run(shards: &[Shard], graph: &PackageGraph, tested: &TestedCrates) -> Vec<(Shard, TestPackages)> {
        let runs = shards.iter().flat_map(Shard::runs);
        runs.map(|run| {
            let packages = Self::around(&run, graph, tested);
            (run, packages)
        })
        .collect()
    }

    pub fn baselines(
        shards: &[Shard],
        graph: &PackageGraph,
        tested: &TestedCrates,
        build: &MutantsBuild,
    ) -> Vec<CommandLine> {
        let distinct: BTreeSet<TestPackages> =
            Self::each_run(shards, graph, tested).into_iter().map(|(_, packages)| packages).collect();
        distinct
            .iter()
            .map(|packages| packages.named(build.line(CargoArgument::Test), CargoArgument::Package))
            .collect()
    }

    pub fn applied_to(&self, line: CommandLine) -> CommandLine {
        self.named(line, MutantsArgument::TestPackage)
    }

    fn named(&self, line: CommandLine, flag: impl fmt::Display + Copy) -> CommandLine {
        self.0.iter().fold(line, |line, name| line.argument(flag).argument(name))
    }
}

#[cfg(test)]
mod tests {
    use super::TestPackages;
    use crate::scope::MutantsBuild;
    use affected::TestedCrates;
    use task::{CargoArgument, CommandLine, Millis, PackageGraph, Program, Shard, SourceFile};
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "log", "manifest_path": "/w/crates/log/Cargo.toml", "dependencies": []},
        {"name": "state", "manifest_path": "/w/crates/state/Cargo.toml", "dependencies": [{"name": "log", "path": "/w/crates/log"}]},
        {"name": "cli", "manifest_path": "/w/crates/cli/Cargo.toml", "dependencies": [{"name": "state", "path": "/w/crates/state"}]}
    ]}"#;
    const STATE: [&str; 3] = ["state", "crates/state/src/lib.rs", "#[cfg(test)] mod tests { #[test] fn t() {} }"];
    const LINE: &str = "gh --test-package log --test-package state";
    const KEPT: &str = include_str!("../../../../.cargo/mutants.toml");
    const SHELL: [&str; 2] = ["sh", "-c"];
    const SLOW: &str = "case \"$*\" in *--no-run*) ;; *) exec sleep 1;; esac";
    const CASES: [(&str, &str, bool); 4] =
        [("false", KEPT, false), ("true", KEPT, true), ("true", "all_features = true", false), (SLOW, KEPT, false)];
    const SHARDS: [&str; 2] = ["log+cli", "log/0/2"];
    const BASELINES: [&str; 2] = ["--all-features --package cli", "--all-features --package log --package state"];

    #[test]
    fn each_run_has_a_baseline_of_its_own_tests_with_the_shards_build_and_timeout_and_a_failing_one_is_refused() {
        let files = [SourceFile::parse(STATE[1], STATE[2].to_owned()).unwrap().in_crate(STATE[0].parse().unwrap())];
        let (graph, tested) = (PackageGraph::from_metadata(METADATA).unwrap(), TestedCrates::collect(&files));
        let shards = SHARDS.map(|shard| shard.parse().unwrap());
        let line = TestPackages::around(&shards[1], &graph, &tested).applied_to(CommandLine::new(Program::GitHub));
        let build = |script, settings| {
            let shell = CommandLine::new(Program::Executable(SHELL[0].into())).arguments([SHELL[1], script]);
            MutantsBuild::sharing(shell, settings, Millis::new(300))
        };
        let gated = CASES.iter().all(|(script, settings, passes)| {
            build(script, settings).and_then(|build| build.passes(&build.line(CargoArgument::Test))).is_ok() == *passes
        });
        let shown = |shards: &[Shard]| TestPackages::baselines(shards, &graph, &tested, &build("", "").unwrap());
        let each = shown(&shards).iter().zip(BASELINES).all(|(line, packages)| line.to_string().ends_with(packages));
        assert!(line.to_string() == LINE && gated && each && shown(&shards).len() == 2 && shown(&[]).is_empty());
    }
}
