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

pub struct TestPackages {
    run: Shard,
    tested_by: BTreeSet<CrateName>,
}

impl TestPackages {
    pub fn around(run: Shard, graph: &PackageGraph, tested: &TestedCrates) -> TestPackages {
        let tested_by = tested.around(graph, &run.crates());
        TestPackages { run, tested_by }
    }

    pub fn each_run(shards: &[Shard], graph: &PackageGraph, tested: &TestedCrates) -> Vec<TestPackages> {
        shards.iter().flat_map(Shard::runs).map(|run| Self::around(run, graph, tested)).collect()
    }

    pub fn baselines(
        shards: &[Shard],
        graph: &PackageGraph,
        tested: &TestedCrates,
        build: &MutantsBuild,
    ) -> Vec<CommandLine> {
        let runs = Self::each_run(shards, graph, tested);
        let distinct: BTreeSet<&BTreeSet<CrateName>> = runs.iter().map(|run| &run.tested_by).collect();
        distinct
            .into_iter()
            .map(|packages| Self::named(packages, build.line(CargoArgument::Test), CargoArgument::Package))
            .collect()
    }

    pub fn run(&self) -> &Shard {
        &self.run
    }

    pub fn applied_to(&self, line: CommandLine) -> CommandLine {
        Self::named(&self.tested_by, self.run.applied_to(line), MutantsArgument::TestPackage)
    }

    fn named(packages: &BTreeSet<CrateName>, line: CommandLine, flag: impl fmt::Display + Copy) -> CommandLine {
        packages.iter().fold(line, |line, name| line.argument(flag).argument(name))
    }
}

#[cfg(test)]
mod tests {
    use super::TestPackages;
    use crate::scope::MutantsBuild;
    use affected::TestedCrates;
    use settings::EmbeddedDefaults;
    use task::{CargoArgument, CommandLine, Interval, PackageGraph, Policy, Program, Shard, SourceFile};
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "log", "manifest_path": "/w/crates/log/Cargo.toml", "dependencies": []},
        {"name": "state", "manifest_path": "/w/crates/state/Cargo.toml", "dependencies": [{"name": "log", "path": "/w/crates/log"}]},
        {"name": "cli", "manifest_path": "/w/crates/cli/Cargo.toml", "dependencies": [{"name": "state", "path": "/w/crates/state"}]}
    ]}"#;
    const STATE: (&str, &str, &str) = ("state", "lib.rs", "#[cfg(test)] mod tests { #[test] fn t() {} }");
    const LINE: &str = "sh -c --package log --shard 0/2 --sharding=round-robin --test-package log --test-package state";
    const KEPT: &str = include_str!("../../../../.cargo/mutants.toml");
    const SHELL: (&str, &str) = ("sh", "-c");
    const PASSING: &str = "true";
    const SLOW: &str = "case \"$*\" in *--no-run*) ;; *) exec sleep 1;; esac";
    const CASES: [(&str, &str, bool); 4] =
        [("false", KEPT, false), (PASSING, KEPT, true), (PASSING, "all_features = true", false), (SLOW, KEPT, false)];
    const SHARDS: [&str; 2] = ["log+cli", "log/0/2"];
    const BASELINES: [&str; 2] = ["--all-features --package cli", "--all-features --package log --package state"];

    #[test]
    fn each_run_has_a_baseline_of_its_own_tests_with_the_shards_build_and_timeout_and_a_failing_one_is_refused() {
        let files = [SourceFile::parse(STATE.1, STATE.2.to_owned()).unwrap().in_crate(STATE.0.parse().unwrap())];
        let (graph, tested) = (PackageGraph::from_metadata(METADATA).unwrap(), TestedCrates::collect(&files));
        let shards = SHARDS.map(|shard| shard.parse::<Shard>().unwrap());
        let shell = CommandLine::new(Program::Executable(SHELL.0.into())).argument(SHELL.1);
        let line = TestPackages::around(shards[1].clone(), &graph, &tested).applied_to(shell.clone());
        let timeout = Policy::defaults().interval(Interval::DiscoverEvery);
        let build = |script, settings| MutantsBuild::sharing(shell.clone().argument(script), settings, timeout);
        let gated = CASES.into_iter().all(|(script, settings, passes)| {
            build(script, settings).and_then(|build| build.passes(&build.line(CargoArgument::Test))).is_ok() == passes
        });
        let shown = |shards: &[Shard]| TestPackages::baselines(shards, &graph, &tested, &build(PASSING, KEPT).unwrap());
        let each = shown(&shards).iter().zip(BASELINES).all(|(line, packages)| line.to_string().ends_with(packages));
        let counted = shown(&shards).len() == BASELINES.len() && shown(&[]).is_empty();
        assert!(line.to_string() == LINE && gated && each && counted);
    }
}
