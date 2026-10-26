use std::fmt;

use proc_macro2::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineNumber(usize);

impl LineNumber {
    pub fn starting(span: Span) -> LineNumber {
        LineNumber(span.start().line)
    }

    pub fn after_index(index: usize) -> LineNumber {
        LineNumber(index + 1)
    }
}

impl fmt::Display for LineNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
