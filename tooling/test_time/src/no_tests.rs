use std::fmt;

pub(super) struct NoTests;

impl fmt::Display for NoTests {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("no tests were found to time")
    }
}
