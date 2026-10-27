mod no_tests;
mod test_binary;
mod test_runs;
mod test_timing;

use std::cmp::Reverse;
use std::time::Duration;

use no_tests::NoTests;
use test_binary::TestBinary;
use test_runs::TestRuns;
use test_timing::TestTiming;

use task::Failure;
use task::Report;
use task::Selection;
use task::Site;
use task::Violation;
use task::Workspace;
use task::{Amount, Ledger, Measure, Millis};
use task::{CargoArgument, CargoTool, CommandLine};
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
                let timings = TestBinary::discover(workspace, selection, policy)?
                    .iter()
                    .flat_map(TestBinary::timings)
                    .collect::<TestRuns>()
                    .passed()?;
                TestTime { timings }.recorded(policy, ledger).verdict()
            },
        )
    }

    fn cargo_test(line: CommandLine, selection: &Selection, policy: &Policy) -> CommandLine {
        line.argument(CargoArgument::Test)
            .arguments(selection.arguments::<CargoTool>())
            .argument(CargoArgument::AllFeatures)
            .argument(CargoArgument::Jobs)
            .argument(policy.compiler_tasks())
            .argument(CargoArgument::NoRun)
            .argument(CargoArgument::Color)
            .argument(CargoArgument::Never)
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
    use super::{TestTime, TestTiming};
    use settings::EmbeddedDefaults;
    use std::path::Path;
    use std::time::Duration;
    use task::{CommandLine, Interval, Ledger, Policy, Program, Report, Selection};
    const RUNS_EVERYTHING: [&str; 2] = ["--no-fail-fast", "test-runner"];
    const SKIPS_DOCTESTS: &str = "--no-run";
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
    fn the_one_cargo_test_run_runs_doctests_and_hands_each_test_binary_to_the_runner() {
        let base = CommandLine::new(Program::Cargo);
        let line = TestTime::cargo_test(base, &Selection::Workspace, &Policy::defaults());
        let shown = line.to_string();
        let whole = RUNS_EVERYTHING.iter().all(|part| shown.contains(part));
        assert!(whole && !shown.contains(SKIPS_DOCTESTS), "{shown}");
    }
}
