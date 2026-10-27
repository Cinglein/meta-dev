use std::fmt;
use std::ops::Add;
use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Micros(u64);

impl Micros {
    const PER_MILLI: u64 = 1000;

    pub fn elapsed_since(origin: Instant) -> Self {
        Self(u64::try_from(origin.elapsed().as_micros()).unwrap_or(u64::MAX))
    }

    pub fn since(self, earlier: Self) -> Self {
        Self(self.0.saturating_sub(earlier.0))
    }
}

impl Add for Micros {
    type Output = Self;

    fn add(self, later: Self) -> Self {
        Self(self.0.saturating_add(later.0))
    }
}

impl fmt::Display for Micros {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown = format!("{}.{:03}ms", self.0 / Self::PER_MILLI, self.0 % Self::PER_MILLI);
        formatter.pad(&shown)
    }
}
