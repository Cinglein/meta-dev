use std::fmt;

use crate::line_number::LineNumber;
use crate::rule::name::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    name: Name,
    line: LineNumber,
}

impl Candidate {
    pub fn new(name: Name, line: LineNumber) -> Candidate {
        Candidate { name, line }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn line(&self) -> LineNumber {
        self.line
    }
}

impl fmt::Display for Candidate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} only carries values into one other file; give it the logic that consumes it", self.name)
    }
}
