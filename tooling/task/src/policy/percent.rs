use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Percent(usize);

impl Percent {
    const WHOLE: usize = 100;

    pub fn applied_to<Whole: Into<usize>, Part: From<usize>>(self, whole: Whole) -> Part {
        Part::from(whole.into() * self.0 / Self::WHOLE)
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
