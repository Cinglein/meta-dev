use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, From, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Most(usize);

impl Most {
    #[must_use]
    pub const fn count(self) -> usize {
        self.0
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
