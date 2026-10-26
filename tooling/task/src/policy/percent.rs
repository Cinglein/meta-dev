use std::num::NonZeroUsize;

use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

use crate::budget::Millis;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, From, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Percent(usize);

impl Percent {
    const WHOLE: usize = 100;

    #[must_use]
    pub const fn new(percent: usize) -> Percent {
        Percent(percent)
    }

    pub fn total(parts: impl IntoIterator<Item = Percent>) -> Option<Percent> {
        parts.into_iter().try_fold(Percent(0), |sum, part| {
            sum.0.checked_add(part.0).map(Percent)
        })
    }

    #[must_use]
    pub const fn is_more_than_whole(self) -> bool {
        self.0 > Self::WHOLE
    }

    pub fn applied_to<Whole: Into<usize>, Part: From<usize>>(self, whole: Whole) -> Part {
        Part::from(whole.into() * self.0 / Self::WHOLE)
    }

    #[must_use]
    pub fn portion_of(self, whole: Millis) -> Millis {
        NonZeroUsize::new(Self::WHOLE).map_or(whole, |all| whole.times(self.0).divided_by(all))
    }

    pub fn exceeds(self, ratio: f64) -> bool {
        let scaled = |count: usize| f64::from(u32::try_from(count).unwrap_or(u32::MAX));
        ratio * scaled(Self::WHOLE) < scaled(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Percent;
    const EIGHTEEN: Percent = Percent(18);
    const FIFTY: usize = 50;
    const NINE: usize = 9;
    const HALF: Percent = Percent(50);
    const QUARTER: f64 = 0.25;
    const ONE_HALF: f64 = 0.5;

    #[test]
    fn a_percent_of_a_whole_rounds_down() {
        assert_eq!(EIGHTEEN.applied_to::<usize, usize>(FIFTY), NINE);
    }

    #[test]
    fn a_percent_exceeds_only_a_smaller_ratio() {
        assert!(HALF.exceeds(QUARTER) && !HALF.exceeds(ONE_HALF));
    }
}
