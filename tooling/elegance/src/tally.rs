use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Mul};

use crate::hundredths::Hundredths;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tally(usize);

impl Tally {
    pub const ONE: Tally = Tally(1);
    pub const PAIR: Tally = Tally(2);
    pub const UPPER_QUARTILE: Tally = Tally(3);
    pub const SHOWN: usize = 10;
    const QUARTERS: usize = 4;

    #[must_use]
    pub fn counting<Item>(items: impl IntoIterator<Item = Item>) -> Tally {
        Tally(items.into_iter().count())
    }

    #[must_use]
    pub fn reaches(self, floor: Tally) -> bool {
        self >= floor
    }

    #[must_use]
    pub fn per(self, whole: Tally) -> Hundredths {
        Hundredths::new((self.0 * Hundredths::WHOLE).checked_div(whole.0).unwrap_or_default())
    }

    #[must_use]
    pub fn quarter_index(self, quarters: Tally) -> Option<usize> {
        (self.0 * quarters.0).div_ceil(Self::QUARTERS).checked_sub(1)
    }

    pub fn quarters() -> impl Iterator<Item = Tally> {
        (0..=Self::QUARTERS).map(Tally)
    }
}

impl Add for Tally {
    type Output = Tally;

    fn add(self, other: Tally) -> Tally {
        Tally(self.0 + other.0)
    }
}

impl Mul for Tally {
    type Output = Tally;

    fn mul(self, other: Tally) -> Tally {
        Tally(self.0 * other.0)
    }
}

impl Sum for Tally {
    fn sum<Tallies: Iterator<Item = Tally>>(tallies: Tallies) -> Tally {
        Tally(tallies.map(|tally| tally.0).sum())
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
