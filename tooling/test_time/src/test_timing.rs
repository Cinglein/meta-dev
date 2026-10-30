use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::clock::Clock;
use task::{CrateName, FailureKind};

use super::test_name::TestName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TestTiming {
    binary: PathBuf,
    test: String,
    elapsed: Duration,
    cpu: Duration,
}

impl TestTiming {
    pub(super) fn new(
        binary: &Path,
        test: impl fmt::Display,
        elapsed: Duration,
        cpu: Duration,
    ) -> TestTiming {
        TestTiming {
            binary: binary.to_path_buf(),
            test: test.to_string(),
            elapsed,
            cpu,
        }
    }

    pub(super) const fn spent(&self, clock: Clock) -> Duration {
        match clock {
            Clock::Wall => self.elapsed,
            Clock::Cpu => self.cpu,
        }
    }

    pub(super) fn failure(&self) -> FailureKind {
        FailureKind::Test {
            crate_name: CrateName::from(self.binary.as_path()),
            name: self.test.as_str().into(),
        }
    }

    pub(super) fn binary(&self) -> &Path {
        &self.binary
    }

    pub(super) fn test(&self) -> &str {
        &self.test
    }
}

impl fmt::Display for TestTiming {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} took {:.1?}, cpu {:.1?}",
            TestName::new(&self.binary, &self.test),
            self.elapsed,
            self.cpu
        )
    }
}
