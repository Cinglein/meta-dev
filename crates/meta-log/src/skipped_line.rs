mod line_number;

use std::fmt;

pub use line_number::LineNumber;

#[derive(Debug)]
pub struct SkippedLine {
    line: LineNumber,
    reason: serde_json::Error,
}

impl SkippedLine {
    pub fn new(line: LineNumber, reason: serde_json::Error) -> Self {
        Self { line, reason }
    }

    pub fn line(&self) -> LineNumber {
        self.line
    }
}

impl fmt::Display for SkippedLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.line, self.reason)
    }
}
