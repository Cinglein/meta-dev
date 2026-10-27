use std::fmt;
use std::num::{NonZeroU64, NonZeroUsize};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::percent::Percent;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Millis(u64);

impl Millis {
    #[must_use]
    pub const fn new(millis: u64) -> Millis {
        Millis(millis)
    }

    #[must_use]
    pub fn rounded_up(elapsed: Duration) -> Millis {
        let nanos_per_milli = u128::from(Duration::from_millis(1).subsec_nanos());
        Millis(u64::try_from(elapsed.as_nanos().div_ceil(nanos_per_milli)).unwrap_or(u64::MAX))
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
        Millis(
            self.0
                .saturating_mul(u64::try_from(count).unwrap_or(u64::MAX)),
        )
    }

    #[must_use]
    pub fn scaled(self, percent: Percent) -> Option<Millis> {
        let [percent, whole] =
            [percent, Percent::ALL].map(|share| u64::try_from(usize::from(share)).ok());
        Some(Millis(self.0.checked_mul(percent?)? / whole?))
    }

    #[must_use]
    pub const fn plus(self, other: Millis) -> Millis {
        Millis(self.0 + other.0)
    }

    #[must_use]
    pub fn checked_minus(self, other: Millis) -> Option<Millis> {
        self.0.checked_sub(other.0).map(Millis)
    }

    #[must_use]
    pub fn times_fitting(self, each: Millis) -> usize {
        usize::try_from(self.0 / each.0.max(1)).unwrap_or(usize::MAX)
    }

    #[must_use]
    pub fn divided_by(self, count: NonZeroUsize) -> Millis {
        Millis(self.0 / u64::try_from(count.get()).unwrap_or(u64::MAX))
    }

    #[must_use]
    pub const fn rounded_up_to(self, step: NonZeroU64) -> Millis {
        let step = step.get();
        Millis(self.0.div_ceil(step) * step)
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
