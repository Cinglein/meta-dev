use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use super::test_run::TestRun;
use super::test_timing::TestTiming;
use task::{Failure, FailureKind, FailureSet};

#[derive(Debug, Default)]
pub(super) struct TestRuns {
    passed: Vec<TestTiming>,
    failed: FailureSet,
    unlisted: Vec<PathBuf>,
}

impl TestRuns {
    pub(super) fn passed(self) -> Result<Vec<TestTiming>, Failure> {
        if self.failed.is_empty() && self.unlisted.is_empty() {
            Ok(self.passed)
        } else {
            Err(Failure::Found {
                printed: self.to_string(),
                found: self.failed,
            })
        }
    }

    pub(super) fn passing(mut self, timing: TestTiming) -> TestRuns {
        self.passed.push(timing);
        self
    }

    pub(super) fn failing(mut self, kind: FailureKind) -> TestRuns {
        self.failed = self.failed.recording(kind);
        self
    }

    pub(super) fn unlisting(mut self, binary: PathBuf) -> TestRuns {
        self.unlisted.push(binary);
        self
    }
}

impl FromIterator<TestRun> for TestRuns {
    fn from_iter<I: IntoIterator<Item = TestRun>>(runs: I) -> TestRuns {
        runs.into_iter()
            .fold(TestRuns::default(), |runs, run| run.filed(runs))
    }
}

impl FromStr for TestRuns {
    type Err = Failure;

    fn from_str(recorded: &str) -> Result<TestRuns, Failure> {
        recorded
            .lines()
            .map(str::parse)
            .collect::<Result<Vec<TestRun>, Failure>>()
            .map(TestRuns::from_iter)
    }
}

impl fmt::Display for TestRuns {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let failed = self.failed.kinds().len();
        write!(
            formatter,
            "{failed} of {} tests failed",
            failed + self.passed.len()
        )?;
        match self.unlisted.len() {
            0 => Ok(()),
            unlisted => write!(
                formatter,
                "; {unlisted} test binaries did not list their tests"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use super::TestRun::{Failed, Passed};
    use super::{TestRun, TestRuns, TestTiming};

    const FAILING: &str = include_str!("../fixtures/failing-test.tsv");
    const FAILING_GOLDEN: &str = include_str!("../fixtures/failing-test.golden.json");

    #[test]
    fn a_failing_test_is_recorded_with_its_crate_and_name() {
        let found = FAILING.parse::<TestRuns>().and_then(TestRuns::passed);
        assert_eq!(
            found
                .err()
                .map(|failure| failure.found().to_string())
                .as_deref(),
            Some(FAILING_GOLDEN)
        );
    }

    const BINARY: &str = "target/debug/deps/bin-1a2b3c";
    const NAMES: [&str; 2] = ["passes", "fails"];
    const TICK: Duration = Duration::from_millis(1);
    const FAILED: &str = "1 of 2 tests failed; 1 test binaries did not list their tests";
    const LINE_BREAK: &str = "\n";
    const SLOWEST: &str = "bin passes took 1.0ms, cpu 1.0ms";
    const BAD: [&str; 3] = ["passed\t1\tsoon\tx\ty", "skipped\tx", "unlisted"];

    #[test]
    fn recorded_runs_read_back_and_a_failing_test_or_an_unlisted_binary_fails_the_one_run() {
        let [passed, failed] =
            NAMES.map(|name| TestTiming::new(Path::new(BINARY), name, TICK, TICK));
        let read = |runs: Vec<TestRun>| {
            let lines: Vec<String> = runs.iter().map(ToString::to_string).collect();
            lines.join(LINE_BREAK).parse().and_then(TestRuns::passed)
        };
        let gone = TestRun::Unlisted(PathBuf::from(BINARY));
        let failing = read(vec![Passed(passed.clone()), Failed(failed), gone]);
        let shown = failing.map_err(|failure| failure.to_string()).err();
        let refused = BAD.iter().all(|line| line.parse::<TestRuns>().is_err());
        let clean = read(vec![Passed(passed.clone())]);
        assert!(refused && shown.as_deref() == Some(FAILED) && passed.to_string() == SLOWEST);
        assert!(clean.is_ok_and(|timings| timings == [passed]));
    }
}
