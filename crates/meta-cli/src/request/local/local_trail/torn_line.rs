use std::fmt;
use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct TornLine {
    line: NonZeroUsize,
    reason: serde_json::Error,
}

impl TornLine {
    pub fn new(line: NonZeroUsize, reason: serde_json::Error) -> Self {
        Self { line, reason }
    }
}

impl fmt::Display for TornLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "skipped worktree record line {}: {}", self.line, self.reason)
    }
}
