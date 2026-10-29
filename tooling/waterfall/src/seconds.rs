use std::cmp::Ordering;
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Sub};

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Deserialize)]
#[serde(transparent)]
pub(crate) struct Seconds(f64);

impl Seconds {
    pub(crate) fn ordered(self, other: Seconds) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Add for Seconds {
    type Output = Seconds;

    fn add(self, other: Seconds) -> Seconds {
        Seconds(self.0 + other.0)
    }
}

impl Sub for Seconds {
    type Output = Seconds;

    fn sub(self, other: Seconds) -> Seconds {
        Seconds(self.0 - other.0)
    }
}

impl Sum for Seconds {
    fn sum<I: Iterator<Item = Seconds>>(seconds: I) -> Seconds {
        seconds.fold(Seconds::default(), Add::add)
    }
}

impl fmt::Display for Seconds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.2}s", self.0)
    }
}
