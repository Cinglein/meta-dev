use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Most(usize);

impl From<usize> for Most {
    fn from(count: usize) -> Most {
        Most(count)
    }
}

impl fmt::Display for Most {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Most {
    #[must_use]
    pub const fn count(self) -> usize {
        self.0
    }

    #[must_use]
    pub fn plus(self, other: Most) -> Option<Most> {
        self.0.checked_add(other.0).map(Most)
    }

    #[must_use]
    pub fn counted<Count: From<usize>>(self) -> Count {
        Count::from(self.0)
    }

    #[must_use]
    pub fn allows(self, count: impl Into<usize>) -> bool {
        count.into() <= self.0
    }
}
