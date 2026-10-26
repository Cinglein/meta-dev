use std::fmt;
use std::iter::Sum;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ByteCount(u64);

impl ByteCount {
    #[must_use]
    pub fn new(bytes: u64) -> Self {
        Self(bytes)
    }
}

impl Sum for ByteCount {
    fn sum<Counts: Iterator<Item = Self>>(counts: Counts) -> Self {
        Self(counts.map(|count| count.0).sum())
    }
}

impl fmt::Display for ByteCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} bytes", self.0)
    }
}
