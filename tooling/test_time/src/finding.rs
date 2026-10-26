use std::fmt;
use std::time::Duration;

use super::test_timing::TestTiming;

pub(super) enum Finding {
    SlowTest {
        timing: TestTiming,
        allowed: Duration,
    },
    SlowSuite {
        total: Duration,
        allowed: Duration,
    },
    NoTests,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::SlowTest { timing, allowed } => {
                write!(formatter, "{timing}, at most {allowed:?} allowed per test")
            }
            Finding::SlowSuite { total, allowed } => write!(
                formatter,
                "the suite took {total:.2?}, at most {allowed:?} allowed"
            ),
            Finding::NoTests => formatter.write_str("no tests were found to time"),
        }
    }
}
