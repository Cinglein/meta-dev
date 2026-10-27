use std::fmt;
use std::iter::Sum;
use std::num::NonZeroUsize;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CoreShare(u64);

impl CoreShare {
    const HUNDRED: u64 = 100;
    const DIGITS: usize = 2;

    pub fn scaled(self, cores: NonZeroUsize) -> Self {
        Self(self.0.saturating_mul(u64::try_from(cores.get()).unwrap_or(u64::MAX)))
    }

    pub fn hundredth(self) -> Self {
        Self(self.0 / Self::HUNDRED)
    }

    pub fn passed_by(self, seen: Self) -> Option<Self> {
        (seen > self).then_some(self)
    }
}

impl FromStr for CoreShare {
    type Err = std::num::ParseIntError;

    fn from_str(cores: &str) -> Result<Self, Self::Err> {
        let (whole, fraction) = cores.trim().split_once('.').unwrap_or((cores.trim(), ""));
        let digits: String = fraction.chars().chain(std::iter::repeat('0')).take(Self::DIGITS).collect();
        Ok(Self(whole.parse::<u64>()?.saturating_mul(Self::HUNDRED).saturating_add(digits.parse::<u64>()?)))
    }
}

impl Sum for CoreShare {
    fn sum<Shares: Iterator<Item = Self>>(shares: Shares) -> Self {
        Self(shares.map(|share| share.0).fold(0, u64::saturating_add))
    }
}

impl fmt::Display for CoreShare {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{:02}", self.0 / Self::HUNDRED, self.0 % Self::HUNDRED)
    }
}
