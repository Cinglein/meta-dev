use std::fmt;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct LineCount(usize);

impl LineCount {
    pub fn len(self) -> usize {
        self.0
    }
}

impl fmt::Display for LineCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} lines", self.0)
    }
}
