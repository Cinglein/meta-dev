use std::fmt;

use super::test_timing::TestTiming;
use task::Failure;

#[derive(Debug)]
pub(super) struct TestRuns {
    passed: Vec<TestTiming>,
    failed: Vec<Failure>,
}

impl TestRuns {
    pub(super) fn passed(self) -> Result<Vec<TestTiming>, Failure> {
        self.failed
            .iter()
            .for_each(|failure| eprintln!("{failure}"));
        if self.failed.is_empty() {
            Ok(self.passed)
        } else {
            Err(Failure::Command(self.to_string()))
        }
    }
}

impl FromIterator<Result<TestTiming, Failure>> for TestRuns {
    fn from_iter<I: IntoIterator<Item = Result<TestTiming, Failure>>>(runs: I) -> TestRuns {
        let (passed, failed): (Vec<_>, Vec<_>) = runs.into_iter().partition(Result::is_ok);
        TestRuns {
            passed: passed.into_iter().flatten().collect(),
            failed: failed.into_iter().filter_map(Result::err).collect(),
        }
    }
}

impl fmt::Display for TestRuns {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} of {} tests",
            self.failed.len(),
            self.failed.len() + self.passed.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::Duration;

    use super::{TestRuns, TestTiming};
    use task::Failure;

    const BINARY: &str = "target/debug/deps/bin-1a2b3c";
    const NAMES: [&str; 2] = ["passes", "fails"];
    const TICK: Duration = Duration::from_millis(1);
    const FAILED: &str = "1 of 2 tests failed";

    #[test]
    fn a_failing_test_fails_the_one_run_and_only_a_clean_run_is_timed() {
        let [passes, fails] = NAMES;
        let timing = || Ok(TestTiming::new(Path::new(BINARY), passes, TICK));
        let failing: TestRuns = [timing(), Err(Failure::Command(fails.to_owned()))]
            .into_iter()
            .collect();
        let shown = failing.passed().map(|timings| timings.len());
        assert_eq!(
            shown.map_err(|failure| failure.to_string()),
            Err(FAILED.to_owned())
        );
        let clean: TestRuns = [timing()].into_iter().collect();
        assert!(clean.passed().is_ok_and(|timings| timings.len() == 1));
    }
}
