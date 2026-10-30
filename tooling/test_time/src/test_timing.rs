use std::fmt;
use std::time::Duration;

use super::clock::Clock;

use super::test_name::TestName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TestTiming {
    name: TestName,
    elapsed: Duration,
    cpu: Duration,
}

impl TestTiming {
    pub(super) fn new(name: TestName, elapsed: Duration, cpu: Duration) -> TestTiming {
        TestTiming { name, elapsed, cpu }
    }

    pub(super) const fn spent(&self, clock: Clock) -> Duration {
        match clock {
            Clock::Wall => self.elapsed,
            Clock::Cpu => self.cpu,
        }
    }

    pub(super) fn name(&self) -> &TestName {
        &self.name
    }
}

impl fmt::Display for TestTiming {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} took {:.1?}, cpu {:.1?}",
            self.name.shown(),
            self.elapsed,
            self.cpu
        )
    }
}
