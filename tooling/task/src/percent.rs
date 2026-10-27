use std::num::NonZeroUsize;

use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};

use crate::budget::Millis;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, From, Into, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Percent(usize);

impl Percent {
    const WHOLE: usize = 100;
    pub const ALL: Percent = Percent(Self::WHOLE);
    pub const NONE: Percent = Percent(0);

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

    pub fn rounded_up<Count: Into<usize>>(part: Count, whole: Count) -> Percent {
        let scaled = part.into().saturating_mul(Self::WHOLE);
        Percent(NonZeroUsize::new(whole.into()).map_or(0, |whole| scaled.div_ceil(whole.get())))
    }

    pub fn applied_to<Whole: Into<usize>, Part: From<usize>>(self, whole: Whole) -> Part {
        Part::from(whole.into().saturating_mul(self.0) / Self::WHOLE)
    }

    pub fn scaled<Whole: Into<usize>, Part: From<usize>>(self, whole: Whole) -> Option<Part> {
        whole
            .into()
            .checked_mul(self.0)
            .map(|scaled| Part::from(scaled / Self::WHOLE))
    }

    #[must_use]
    pub fn portion_of(self, whole: Millis) -> Millis {
        NonZeroUsize::new(Self::WHOLE).map_or(whole, |all| whole.times(self.0).divided_by(all))
    }

    #[must_use]
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
    const FIFTY_NINE: usize = 59;
    const NOTHING: usize = 0;
    const SIXTEEN: Percent = Percent(16);
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

    #[test]
    fn a_share_rounds_up_and_a_share_of_nothing_is_none() {
        assert_eq!(Percent::rounded_up(NINE, FIFTY), EIGHTEEN);
        assert_eq!(Percent::rounded_up(NINE, FIFTY_NINE), SIXTEEN);
        assert_eq!(Percent::rounded_up(NINE, NOTHING), Percent::NONE);
    }
}
