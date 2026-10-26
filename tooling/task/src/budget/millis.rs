use std::fmt;
use std::num::{NonZeroU64, NonZeroUsize};
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Millis(u64);

impl Millis {
    const HALVING: u64 = 2;

    #[must_use]
    pub const fn new(millis: u64) -> Millis {
        Millis(millis)
    }

    #[must_use]
    pub const fn duration(self) -> Duration {
        Duration::from_millis(self.0)
    }

    #[must_use]
    pub const fn whole_seconds(self) -> u64 {
        self.duration().as_secs()
    }

    #[must_use]
    pub fn times(self, count: usize) -> Millis {
        Millis(self.0 * u64::try_from(count).unwrap_or(u64::MAX))
    }

    #[must_use]
    pub const fn plus(self, other: Millis) -> Millis {
        Millis(self.0 + other.0)
    }

    #[must_use]
    pub fn div_ceil(self, other: Millis) -> usize {
        usize::try_from(self.0.div_ceil(other.0.max(1))).unwrap_or(usize::MAX)
    }

    #[must_use]
    pub fn divided_by(self, count: NonZeroUsize) -> Millis {
        Millis(self.0 / u64::try_from(count.get()).unwrap_or(u64::MAX))
    }

    #[must_use]
    pub const fn rounded_to(self, step: NonZeroU64) -> Millis {
        let step = step.get();
        Millis((self.0 + step / Self::HALVING) / step * step)
    }
}

impl fmt::Display for Millis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Millis;
    #[test]
    fn whole_seconds_drops_the_fraction_of_a_second() {
        assert_eq!(Millis::new(64).times(32).whole_seconds(), 2);
    }
}
