use std::fmt;

pub(super) struct NoTests;

impl fmt::Display for NoTests {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "no tests were found to time; a test runner set in the project's cargo config may have taken precedence",
        )
    }
}
