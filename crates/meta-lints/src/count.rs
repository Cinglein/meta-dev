use std::fmt;
use std::iter::Sum;
use std::ops::Add;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Count(usize);

impl Count {
    pub const ONE: Count = Count(1);

    pub fn of_len(len: usize) -> Count {
        Count(len)
    }
}

impl Add for Count {
    type Output = Count;

    fn add(self, other: Count) -> Count {
        Count(self.0 + other.0)
    }
}

impl Sum for Count {
    fn sum<Counts: Iterator<Item = Count>>(counts: Counts) -> Count {
        counts.fold(Count::default(), Add::add)
    }
}

impl<'count> Sum<&'count Count> for Count {
    fn sum<Counts: Iterator<Item = &'count Count>>(counts: Counts) -> Count {
        counts.copied().sum()
    }
}

impl fmt::Display for Count {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
