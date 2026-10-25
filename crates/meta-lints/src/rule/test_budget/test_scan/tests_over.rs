use std::fmt;

use super::super::amount::Amount;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TestsOver {
    tests: Amount,
    limit: Amount,
}

impl TestsOver {
    pub(super) fn new(tests: Amount, limit: Amount) -> TestsOver {
        TestsOver { tests, limit }
    }
}

impl fmt::Display for TestsOver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} tests, at most {} allowed", self.tests, self.limit)
    }
}
