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

    pub fn baselines(runs: &[TestPackages], build: &MutantsBuild) -> Vec<Option<CommandLine>> {
        let first =
            |index, run: &TestPackages| runs.iter().position(|other| other.tested_by == run.tested_by) == Some(index);
        runs.iter()
            .enumerate()
            .map(|(index, run)| {
                first(index, run)
                    .then(|| Self::named(&run.tested_by, build.line(CargoArgument::Test), CargoArgument::Package))
            })
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
    use crate::scope::{MutantsBuild, MutantsRun, test_packages::TestPackages};
    use affected::TestedCrates;
    use settings::EmbeddedDefaults;
    use std::path::PathBuf;
    use task::{CargoArgument, CommandLine, Failure, Interval, MemoryShare, PackageGraph, Policy};
    use task::{Program, Shard, SourceFile, Stdout};
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "log", "manifest_path": "/w/crates/log/Cargo.toml", "dependencies": []},
        {"name": "state", "manifest_path": "/w/crates/state/Cargo.toml", "dependencies": [{"name": "log", "path": "/w/crates/log"}]},
        {"name": "cli", "manifest_path": "/w/crates/cli/Cargo.toml", "dependencies": [{"name": "state", "path": "/w/crates/state"}]}
    ]}"#;
    const STATE: (&str, &str, &str) = ("state", "lib.rs", "#[cfg(test)] mod tests { #[test] fn t() {} }");
    const LINE: &str = "sh -c --package log --shard 0/2 --sharding=round-robin --test-package log --test-package state";
    const KEPT: &str = include_str!("../../../../.cargo/mutants.toml");
    const SH: (&str, &str) = ("sh", "-c");
    const PASSING: &str = "true";
    const UNJUDGED: &str = "exit 1";
    const SLOW: &str = "exec sleep 1";
    const ORPHAN: &str = "meta-orphan";
    const ORPHANING: &str = "case \"$*\" in *--no-run*) ;; *) set -m; sleep 5 & echo $! > \"${TMPDIR:-/tmp}/meta-orphan.$PPID\"; exec sleep 1;; esac";
    const LEFT: &str = "f=\"${TMPDIR:-/tmp}/meta-orphan.$PPID\"; test -s \"$f\" && sleep 0.1 && ! ps -o stat= -p \"$(cat \"$f\")\" | grep -qv Z";
    const FAILING: [(&str, &str); 4] = [("false", KEPT), (PASSING, "features = []"), (SLOW, KEPT), (ORPHANING, KEPT)];
    const GRANTED: MemoryShare = MemoryShare::Granted;
    const STARVED: MemoryShare = MemoryShare::Reserve;
    const SHARDS: [&str; 2] = ["log+cli", "log/0/2"];
    const BASELINES: [&str; 2] = ["--all-features --package cli", "--all-features --package log --package state"];
    #[test]
    fn a_runs_baseline_is_its_own_tests_once_with_the_shards_build_and_limits_and_a_failing_one_ends_all_it_started() {
        let files = [SourceFile::parse(STATE.1, STATE.2.to_owned()).unwrap().in_crate(STATE.0.parse().unwrap())];
        let (graph, tested) = (PackageGraph::from_metadata(METADATA).unwrap(), TestedCrates::collect(&files));
        let runs = TestPackages::each_run(&SHARDS.map(|shard| shard.parse::<Shard>().unwrap()), &graph, &tested);
        let (policy, sh) = (Policy::defaults(), CommandLine::new(Program::Executable(SH.0.into())).argument(SH.1));
        let timeout = policy.interval(Interval::DiscoverEvery);
        let build = |script, kept| MutantsBuild::sharing(sh.clone().argument(script), kept, timeout, timeout);
        let passes = |build: &MutantsBuild, share| {
            let run = MutantsRun::new(sh.clone().argument(UNJUDGED), PathBuf::new())
                .baselined(Some(build.line(CargoArgument::Test)));
            run.accepted(build, policy.memory(share), &policy)
        };
        let failed = FAILING.map(|(script, kept)| build(script, kept).and_then(|made| passes(&made, GRANTED)).is_err());
        let ended = sh.clone().argument(LEFT).captured::<Stdout>().is_ok();
        std::fs::remove_file(std::env::temp_dir().join(ORPHAN).with_extension(std::process::id().to_string())).unwrap();
        let starved = passes(&build(SLOW, KEPT).unwrap(), STARVED);
        let shown = TestPackages::baselines(&runs, &build(PASSING, KEPT).unwrap());
        let each = shown.iter().flatten().zip(BASELINES).all(|(line, packages)| line.to_string().ends_with(packages));
        let once = shown.iter().flatten().count() == BASELINES.len() && shown.len() > BASELINES.len();
        assert!(runs.last().is_some_and(|run| run.applied_to(sh.clone()).to_string() == LINE) && each && once);
        assert!(passes(&build(PASSING, KEPT).unwrap(), GRANTED).is_ok() && !failed.contains(&false) && ended);
        assert!(matches!(starved, Err(Failure::BudgetEnforced { .. })));
    }
}
