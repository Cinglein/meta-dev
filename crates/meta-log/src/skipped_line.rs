mod line_number;

use std::fmt;

use crate::detail::Detail;
pub use line_number::LineNumber;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkippedLine {
    line: LineNumber,
    reason: Detail,
}

impl SkippedLine {
    pub fn new(line: LineNumber, reason: Detail) -> Self {
        Self { line, reason }
    }
}

impl fmt::Display for SkippedLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.line, self.reason)
    }
}
