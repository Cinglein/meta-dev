use std::fmt;
use std::iter::Sum;
use std::ops::Add;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(transparent)]
pub struct ItemCount(usize);

impl ItemCount {
    pub const ONE: ItemCount = ItemCount(1);

    pub fn of_len(len: usize) -> ItemCount {
        ItemCount(len)
    }

    pub fn excess_len(self, limit: ItemCount) -> usize {
        self.0.saturating_sub(limit.0)
    }
}

impl Add for ItemCount {
    type Output = ItemCount;

    fn add(self, other: ItemCount) -> ItemCount {
        ItemCount(self.0 + other.0)
    }
}

impl Sum for ItemCount {
    fn sum<Counts: Iterator<Item = ItemCount>>(counts: Counts) -> ItemCount {
        counts.fold(ItemCount::default(), Add::add)
    }
}

impl fmt::Display for ItemCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
