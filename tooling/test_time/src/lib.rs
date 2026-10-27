mod cargo_test;
mod no_tests;
mod runner_config;
mod test_run;
mod test_runner;
mod test_runs;
mod test_timing;

use std::cmp::Reverse;
use std::fs;
use std::time::Duration;

use cargo_test::CargoTest;
use no_tests::NoTests;
use runner_config::RunnerConfig;
pub use test_runner::TestRunner;
use test_runs::TestRuns;
use test_timing::TestTiming;

use task::Failure;
use task::Program;
use task::Report;
use task::Selection;
use task::Site;
use task::Violation;
use task::Workspace;
use task::{Amount, Ledger, Measure, Millis};
use task::{Limit, Policy};

#[derive(Debug)]
pub struct TestTime {
    timings: Vec<TestTiming>,
}

impl TestTime {
    pub fn run(
        workspace: &Workspace,
        selection: &Selection,
        policy: &Policy,
        ledger: &mut Ledger,
    ) -> Result<(), Failure> {
        selection.reaching().map_or_else(
            || {
                println!("test time: no crate affected");
                Ok(())
            },
            |selection| {
                let runs = tempfile::NamedTempFile::new().map_err(|error| Failure::Io {
                    site: Site::Workspace,
                    error,
                })?;
                let cargo =
                    |selection| CargoTest::new(workspace.line(Program::Cargo), selection, policy);
                let ran = cargo(selection)
                    .through(&RunnerConfig::current()?)
                    .environment(TestRunner::RUNS, runs.path())
                    .run();
                let documented = selection.within(workspace.packages().doctested());
                let documented_ran = documented
                    .reaching()
                    .map_or(Ok(()), |documented| cargo(documented).doctests().run());
                let timings = fs::read_to_string(runs.path())
                    .map_err(|error| Failure::Io {
                        site: Site::File(runs.path().to_path_buf()),
                        error,
                    })?
                    .parse::<TestRuns>()?
                    .passed()?;
                ran.and(documented_ran)?;
                TestTime { timings }.recorded(policy, ledger).verdict()
            },
        )
    }

    fn recorded(mut self, policy: &Policy, ledger: &mut Ledger) -> Report {
        self.timings.sort_by_key(|timing| Reverse(timing.elapsed()));
        let total: Duration = self.timings.iter().map(TestTiming::elapsed).sum();
        println!(
            "test time: {} tests in {total:.2?}, slowest {}:",
            self.timings.len(),
            policy.limit(Limit::SlowestShown).count()
        );
        self.timings
            .iter()
            .take(policy.limit(Limit::SlowestShown).count())
            .for_each(|timing| println!("  {timing}"));
        let slowest = self
            .timings
            .first()
            .map_or(Duration::ZERO, TestTiming::elapsed);
        ledger.spend(
            Measure::TestWallTime,
            Amount::Duration(Millis::rounded_up(total)),
        );
        ledger.spend(
            Measure::SlowestTest,
            Amount::Duration(Millis::rounded_up(slowest)),
        );
        let none = self
            .timings
            .is_empty()
            .then(|| Violation::new(Site::Workspace, NoTests));
        Report::new("no tests timed", none.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{CargoTest, RunnerConfig, TestTime, TestTiming};
    use settings::EmbeddedDefaults;
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use task::{CommandLine, Interval, Ledger, Policy, Program, Report, Selection};
    const RUNNER: &str = "target/debug/x\"task";
    const RUNNER_KEY: &str = "--config";
    const DOCTESTS: &str = "--doc";
    const RUNS_EVERYTHING: [&str; 3] = [
        "--no-fail-fast --tests --config",
        "target.'cfg(all())'.runner = [\"target/debug/x\\\"task\", \"test-runner\"]",
        "--workspace",
    ];
    const NAMES: [&str; 4] = ["target/debug/deps/bin-1a2b3c", "exact", "slow", "bulk"];
    const TICK: Duration = Duration::from_millis(1);
    const WHOLE_IN_UNITS: usize = 4;
    const REPORTED: [&str; 2] = ["slowest-test", "test-time"];
    const NOTHING_TIMED: [&str; 1] = ["no tests were found to time"];
    const HEADLINE: &str = "timed";

    impl TestTime {
        fn over_budget(timings: Vec<TestTiming>, policy: &Policy) -> Report {
            let mut ledger = Ledger::default();
            let found = TestTime { timings }.recorded(policy, &mut ledger);
            let verdict = ledger.statement(policy).verdict(&Selection::Workspace);
            Report::new(
                HEADLINE,
                [found.violations(), verdict.violations()].concat(),
            )
        }
    }

    #[test]
    fn a_test_over_its_allowance_and_a_suite_over_its_total_are_over_budget_and_the_limits_pass() {
        let policy = Policy::defaults();
        let limit = policy.interval(Interval::PerTest).duration();
        let [binary, exact, slow, bulk] = NAMES;
        let timing = |name: &str, elapsed| TestTiming::new(Path::new(binary), name, elapsed);
        let exact = (0..WHOLE_IN_UNITS).map(|_| timing(exact, limit)).collect();
        let total = policy.interval(Interval::Total).duration();
        let over = TestTime::over_budget(
            vec![timing(slow, limit + TICK), timing(bulk, total)],
            &policy,
        );
        assert!(
            TestTime::over_budget(exact, &policy).is_clean() && over.lists_exactly(&REPORTED),
            "{over}"
        );
        assert!(TestTime::over_budget(Vec::new(), &policy).lists_exactly(&NOTHING_TIMED));
    }

    #[test]
    fn libtest_targets_go_through_the_runner_and_doctests_run_without_it() {
        let policy = Policy::defaults();
        let cargo = || {
            CargoTest::new(
                CommandLine::new(Program::Cargo),
                &Selection::Workspace,
                &policy,
            )
        };
        let shown = cargo()
            .through(&RunnerConfig::new(PathBuf::from(RUNNER)))
            .to_string();
        let docs = cargo().doctests().to_string();
        let through = RUNS_EVERYTHING.iter().all(|part| shown.contains(part));
        let plain = docs.ends_with(DOCTESTS) && !docs.contains(RUNNER_KEY);
        assert!(
            through && plain && !shown.contains(DOCTESTS),
            "{shown}\n{docs}"
        );
    }
}
