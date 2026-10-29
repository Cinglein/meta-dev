use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Sub};
use std::time::SystemTime;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Micros(u64);

impl Micros {
    const PER_MILLI: u64 = 1000;
    const TENTHS: u64 = 100;

    #[must_use]
    pub fn now() -> Self {
        let since = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
        Self(u64::try_from(since.as_micros()).unwrap_or(u64::MAX))
    }

    #[must_use]
    pub const fn new(micros: u64) -> Self {
        Self(micros)
    }

    #[must_use]
    pub fn micros(self) -> u64 {
        self.0
    }
}

impl Add for Micros {
    type Output = Self;

    fn add(self, later: Self) -> Self {
        Self(self.0.saturating_add(later.0))
    }
}

impl Sub for Micros {
    type Output = Self;

    fn sub(self, earlier: Self) -> Self {
        Self(self.0.saturating_sub(earlier.0))
    }
}

impl Sum for Micros {
    fn sum<All: Iterator<Item = Self>>(all: All) -> Self {
        all.fold(Self::default(), Add::add)
    }
}

impl fmt::Display for Micros {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown = format!("{}.{}ms", self.0 / Self::PER_MILLI, self.0 % Self::PER_MILLI / Self::TENTHS);
        formatter.pad(&shown)
    }
}
