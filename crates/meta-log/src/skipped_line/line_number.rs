use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineNumber(usize);

impl LineNumber {
    pub fn from_index(index: usize) -> Self {
        Self(index + 1)
    }
}

impl fmt::Display for LineNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "line {}", self.0)
    }
}
