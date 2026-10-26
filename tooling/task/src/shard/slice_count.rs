use std::num::NonZeroUsize;
use std::ops::Range;

use derive_more::{Display, From, FromStr, Into};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, From, FromStr, Into)]
pub struct SliceCount(NonZeroUsize);

impl SliceCount {
    #[must_use]
    pub const fn index_range(self) -> Range<usize> {
        0..self.0.get()
    }
}
