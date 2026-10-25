use std::fmt;

use super::super::super::measurement::Measurement;
use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TestOver {
    test: Name,
    measurement: Measurement,
}

impl TestOver {
    pub(super) fn new(test: Name, measurement: Measurement) -> TestOver {
        TestOver { test, measurement }
    }
}

impl fmt::Display for TestOver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fn {} {}", self.test, self.measurement)
    }
}
