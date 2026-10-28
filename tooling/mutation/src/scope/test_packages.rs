use std::collections::BTreeSet;
use std::fmt;

use super::MutantsBuild;
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

    pub fn passes_baseline(&self, build: &MutantsBuild) -> Result<(), Failure> {
        build.passes(&self.named(build.line(CargoArgument::Test), CargoArgument::Package))
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
    use task::{CommandLine, Millis, PackageGraph, Program, SourceFile};
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
    fn a_failing_unmutated_run_of_the_tests_around_the_shards_with_their_build_settings_is_refused() {
        let files = FILES.map(|(name, path, text)| {
            SourceFile::parse(path, text.to_owned()).unwrap().in_crate(name.parse().unwrap())
        });
        let graph = PackageGraph::from_metadata(METADATA).unwrap();
        let packages = TestPackages::around(&"log".parse().unwrap(), &graph, &TestedCrates::collect(&files));
        let line = packages.applied_to(CommandLine::new(Program::GitHub));
        let gated = CASES.iter().all(|(cargo, settings, passes)| {
            let cargo = CommandLine::new(Program::Executable(cargo[0].into())).arguments(&cargo[1..]);
            MutantsBuild::sharing(cargo, settings, TIMEOUT).and_then(|build| packages.passes_baseline(&build)).is_ok()
                == *passes
        });
        assert!(line.to_string() == LINE && gated);
    }
}
