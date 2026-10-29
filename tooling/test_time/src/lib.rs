mod cargo_build;
mod cargo_test;
mod children_cpu;
mod clock;
mod no_tests;
mod reference_cpu;
mod runner_config;
mod test_run;
mod test_runner;
mod test_runs;
mod test_timing;

use std::cmp::Reverse;
use std::fs;
use std::time::Duration;

use cargo_build::CargoBuild;
use cargo_test::CargoTest;
use clock::Clock;
use git_trace::GitProcesses;
use no_tests::NoTests;
use reference_cpu::ReferenceCpu;
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
    reference: ReferenceCpu,
    git: GitProcesses,
}

impl TestTime {
    const GIT_SETTINGS: [(&'static str, &'static str); 13] = [
        ("GIT_CONFIG_COUNT", "6"),
        ("GIT_CONFIG_KEY_0", "core.fsync"),
        ("GIT_CONFIG_VALUE_0", "none"),
        ("GIT_CONFIG_KEY_1", "gc.auto"),
        ("GIT_CONFIG_VALUE_1", "0"),
        ("GIT_CONFIG_KEY_2", "core.looseCompression"),
        ("GIT_CONFIG_VALUE_2", "0"),
        ("GIT_CONFIG_KEY_3", "pack.compression"),
        ("GIT_CONFIG_VALUE_3", "0"),
        ("GIT_CONFIG_KEY_4", "pack.threads"),
        ("GIT_CONFIG_VALUE_4", "1"),
        ("GIT_CONFIG_KEY_5", "core.preloadIndex"),
        ("GIT_CONFIG_VALUE_5", "false"),
    ];

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
                let unmade = |error| Failure::Io {
                    site: Site::Workspace,
                    error,
                };
                let runs = tempfile::NamedTempFile::new().map_err(unmade)?;
                let traces = tempfile::tempdir().map_err(unmade)?;
                let line = || workspace.line(Program::Cargo);
                selection
                    .within(workspace.packages().with_examples())
                    .reaching()
                    .map_or(Ok(()), |examples| {
                        CargoBuild::examples(line(), examples, policy).run()
                    })?;
                let reference = ReferenceCpu::measured(policy)?;
                let cargo = |selection| CargoTest::new(line(), selection, policy);
                let line = cargo(selection)
                    .through(&RunnerConfig::current()?)
                    .environment(TestRunner::RUNS, runs.path())
                    .environment(GitProcesses::TARGET, traces.path());
                let ran = Self::GIT_SETTINGS
                    .into_iter()
                    .fold(line, |line, (name, value)| line.environment(name, value))
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
                let git = GitProcesses::traced(traces.path())?;
                TestTime {
                    timings,
                    reference,
                    git,
                }
                .recorded(policy, ledger)
                .verdict()
            },
        )
    }

    fn recorded(mut self, policy: &Policy, ledger: &mut Ledger) -> Report {
        self.timings
            .sort_by_key(|timing| Reverse(timing.spent(Clock::Wall)));
        let [total, cpu] = [Clock::Wall, Clock::Cpu].map(|clock| self.spent(clock));
        println!(
            "test time: {} tests in {total:.2?}, cpu {cpu:.2?} ({}), slowest {}:",
            self.timings.len(),
            self.reference,
            policy.limit(Limit::SlowestShown).count()
        );
        self.timings
            .iter()
            .take(policy.limit(Limit::SlowestShown).count())
            .for_each(|timing| println!("  {timing}"));
        println!("{}", self.git);
        ledger.spend(Measure::GitProcesses, self.git.amount());
        let slowest = self
            .timings
            .first()
            .map_or(Duration::ZERO, |timing| timing.spent(Clock::Wall));
        ledger.spend(
            Measure::TestWallTime,
            Amount::Duration(Millis::rounded_up(total)),
        );
        ledger.spend(
            Measure::TestCpuTime,
            Amount::Duration(Millis::rounded_up(cpu)),
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

    fn spent(&self, clock: Clock) -> Duration {
        self.timings.iter().map(|timing| timing.spent(clock)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CargoBuild, CargoTest, GitProcesses, ReferenceCpu, RunnerConfig, TestTime, TestTiming,
    };
    use settings::EmbeddedDefaults;
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use task::{CommandLine, Interval, Ledger, Policy, Program, Report, Selection};
    const RUNNER: &str = "target/debug/x\"task";
    const RUNNER_KEY: &str = "--config";
    const DOCTESTS: &str = "--doc";
    const BUILD: &str = " build --workspace ";
    const EXAMPLES: &str = "--examples --profile test";
    const THROUGH: &str = "--no-fail-fast --tests --config target.'cfg(all())'.runner = [\"target/debug/x\\\"task\", \"test-runner\"]";
    const NAMES: [&str; 4] = ["target/debug/deps/bin-1a2b3c", "exact", "slow", "bulk"];
    const TICK: Duration = Duration::from_millis(1);
    const FAST_RUNNER_SECS: f64 = 0.762;
    const RAW_CPU_SECS: f64 = 2.85;
    const WHOLE_IN_UNITS: usize = 4;
    const REPORTED: [&str; 3] = ["slowest-test", "test-cpu-time", "test-time"];
    const NOTHING_TIMED: [&str; 1] = ["no tests were found to time"];
    const HEADLINE: &str = "timed";

    impl TestTime {
        fn over_budget(timings: Vec<TestTiming>, policy: &Policy) -> Report {
            let mut ledger = Ledger::default();
            let reference = ReferenceCpu::new(Duration::from_secs_f64(FAST_RUNNER_SECS));
            let git = GitProcesses::default();
            let found = TestTime {
                timings,
                reference,
                git,
            }
            .recorded(policy, &mut ledger);
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
        let raw = Duration::from_secs_f64(RAW_CPU_SECS);
        let fast = vec![TestTiming::new(Path::new(binary), slow, TICK, raw)];
        let timing =
            |name: &str, elapsed| TestTiming::new(Path::new(binary), name, elapsed, elapsed);
        let exact = (0..WHOLE_IN_UNITS).map(|_| timing(exact, limit)).collect();
        let total = policy.interval(Interval::Total).duration();
        let over = TestTime::over_budget(
            vec![timing(slow, limit + TICK), timing(bulk, total)],
            &policy,
        );
        let ok = |run| TestTime::over_budget(run, &policy).is_clean();
        assert!(ok(exact) && ok(fast) && over.lists_exactly(&REPORTED));
        assert!(TestTime::over_budget(Vec::new(), &policy).lists_exactly(&NOTHING_TIMED));
    }

    #[test]
    fn libtest_targets_go_through_the_runner_and_doctests_run_without_it() {
        let (policy, base) = (Policy::defaults(), CommandLine::new(Program::Cargo));
        let cargo = |line| CargoTest::new(line, &Selection::Workspace, &policy);
        let runner = RunnerConfig::new(PathBuf::from(RUNNER));
        let shown = cargo(base.clone()).through(&runner).to_string();
        let docs = cargo(base.clone()).doctests().to_string();
        let built = CargoBuild::examples(base, &Selection::Workspace, &policy).to_string();
        let plain = docs.ends_with(DOCTESTS) && !docs.contains(RUNNER_KEY);
        let through = shown.ends_with(THROUGH) && !shown.contains(DOCTESTS);
        let examples = built.contains(BUILD) && built.ends_with(EXAMPLES);
        assert!(through && plain && examples, "{shown}\n{docs}\n{built}");
    }
}
