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
        let all: TestPackages =
            Self::each_run(shards, graph, tested).into_iter().map(|(_, packages)| packages).collect();
        Some(all)
            .filter(|all| !all.0.is_empty())
            .map(|all| all.named(build.line(CargoArgument::Test), CargoArgument::Package))
            .into_iter()
            .collect()
    }

    pub fn applied_to(&self, line: CommandLine) -> CommandLine {
        self.named(line, MutantsArgument::TestPackage)
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
    use crate::scope::MutantsBuild;
    use affected::TestedCrates;
    use task::{CargoArgument, CommandLine, Millis, PackageGraph, Program, SourceFile};
    const METADATA: &str = r#"{"workspace_root": "/w", "packages": [
        {"name": "log", "manifest_path": "/w/crates/log/Cargo.toml", "dependencies": []},
        {"name": "state", "manifest_path": "/w/crates/state/Cargo.toml", "dependencies": [{"name": "log", "path": "/w/crates/log"}]},
        {"name": "cli", "manifest_path": "/w/crates/cli/Cargo.toml", "dependencies": [{"name": "state", "path": "/w/crates/state"}]}
    ]}"#;
    const STATE: &str = "#[cfg(test)] mod tests { #[test] fn t() {} }";
    const FILES: [(&str, &str, &str); 2] =
        [("state", "crates/state/src/lib.rs", STATE), ("cli", "crates/cli/src/lib.rs", "pub struct S;")];
    const LINE: &str = "gh --test-package log --test-package state";
    const KEPT: &str = include_str!("../../../../.cargo/mutants.toml");
    const SLOW: &[&str] = &["sh", "-c", "case \"$*\" in *--no-run*) ;; *) exec sleep 1;; esac"];
    const CASES: [(&[&str], &str, bool); 4] = [
        (&["false"], KEPT, false),
        (&["true"], KEPT, true),
        (&["true"], "all_features = true", false),
        (SLOW, KEPT, false),
    ];
    const TIMEOUT: Millis = Millis::new(300);

    #[test]
    fn each_run_has_a_baseline_of_its_own_tests_with_the_shards_build_and_timeout_and_a_failing_one_is_refused() {
        let files = FILES.map(|(name, path, text)| {
            SourceFile::parse(path, text.to_owned()).unwrap().in_crate(name.parse().unwrap())
        });
        let (graph, tested) = (PackageGraph::from_metadata(METADATA).unwrap(), TestedCrates::collect(&files));
        let line = TestPackages::around(&"log".parse().unwrap(), &graph, &tested)
            .applied_to(CommandLine::new(Program::GitHub));
        let build = |cargo: &[&str], settings| {
            let cargo = CommandLine::new(Program::Executable(cargo[0].into())).arguments(&cargo[1..]);
            MutantsBuild::sharing(cargo, settings, TIMEOUT)
        };
        let gated = CASES.iter().all(|(cargo, settings, passes)| {
            build(cargo, settings).and_then(|build| build.passes(&build.line(CargoArgument::Test))).is_ok() == *passes
        });
        let none = TestPackages::baselines(&[], &graph, &tested, &build(&["gh"], "").unwrap());
        assert!(line.to_string() == LINE && gated && none.is_empty());
    }
}
