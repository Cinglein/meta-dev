use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use super::test_runs::TestRuns;
use super::test_timing::TestTiming;
use task::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TestRun {
    Passed(TestTiming),
    Failed(TestTiming),
    Unlisted(PathBuf),
}

impl TestRun {
    const PASSED: &str = "passed";
    const FAILED: &str = "failed";
    const UNLISTED: &str = "unlisted";
    const SEPARATOR: char = '\t';

    pub(super) fn filed(self, runs: TestRuns) -> TestRuns {
        match self {
            TestRun::Passed(timing) => runs.passing(timing),
            TestRun::Failed(timing) => runs.failing(timing.test()),
            TestRun::Unlisted(binary) => runs.unlisting(binary),
        }
    }

    fn timed(nanos: &str, test: &str, binary: &str) -> Option<TestTiming> {
        nanos
            .parse()
            .ok()
            .map(|nanos| TestTiming::new(Path::new(binary), test, Duration::from_nanos(nanos)))
    }

    fn written(
        formatter: &mut fmt::Formatter<'_>,
        outcome: &str,
        timing: &TestTiming,
    ) -> fmt::Result {
        let separator = Self::SEPARATOR;
        write!(
            formatter,
            "{outcome}{separator}{}{separator}{}{separator}{}",
            timing.elapsed().as_nanos(),
            timing.test(),
            timing.binary().display()
        )
    }
}

impl fmt::Display for TestRun {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestRun::Passed(timing) => Self::written(formatter, Self::PASSED, timing),
            TestRun::Failed(timing) => Self::written(formatter, Self::FAILED, timing),
            TestRun::Unlisted(binary) => {
                write!(
                    formatter,
                    "{}{}{}",
                    Self::UNLISTED,
                    Self::SEPARATOR,
                    binary.display()
                )
            }
        }
    }
}

impl FromStr for TestRun {
    type Err = Failure;

    fn from_str(line: &str) -> Result<TestRun, Failure> {
        let fields: Vec<&str> = line.split(Self::SEPARATOR).collect();
        let run = match fields.as_slice() {
            [Self::UNLISTED, binary] => Some(TestRun::Unlisted(PathBuf::from(binary))),
            [Self::PASSED, nanos, test, binary] => {
                Self::timed(nanos, test, binary).map(TestRun::Passed)
            }
            [Self::FAILED, nanos, test, binary] => {
                Self::timed(nanos, test, binary).map(TestRun::Failed)
            }
            _ => None,
        };
        run.ok_or_else(|| Failure::Command(format!("{line} is not a recorded test run")))
    }
}
