use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableFailure(String);

impl TableFailure {
    pub fn new(failure: impl fmt::Display) -> Self {
        Self(failure.to_string())
    }
}

impl fmt::Display for TableFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
