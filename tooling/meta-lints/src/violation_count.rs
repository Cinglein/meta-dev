use std::fmt;
use std::iter::Sum;
use std::ops::Add;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViolationCount(usize);

impl ViolationCount {
    pub const ONE: ViolationCount = ViolationCount(1);

    pub fn of_len(len: usize) -> ViolationCount {
        ViolationCount(len)
    }
}

impl Add for ViolationCount {
    type Output = ViolationCount;

    fn add(self, other: ViolationCount) -> ViolationCount {
        ViolationCount(self.0 + other.0)
    }
}

impl Sum for ViolationCount {
    fn sum<Counts: Iterator<Item = ViolationCount>>(counts: Counts) -> ViolationCount {
        counts.fold(ViolationCount::default(), Add::add)
    }
}

impl<'count> Sum<&'count ViolationCount> for ViolationCount {
    fn sum<Counts: Iterator<Item = &'count ViolationCount>>(counts: Counts) -> ViolationCount {
        counts.copied().sum()
    }
}

impl fmt::Display for ViolationCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
