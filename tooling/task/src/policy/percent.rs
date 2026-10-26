use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Percent(usize);

impl Percent {
    const WHOLE: usize = 100;

    pub fn applied_to<Whole: Into<usize>, Part: From<usize>>(self, whole: Whole) -> Part {
        Part::from(whole.into() * self.0 / Self::WHOLE)
    }
}

#[cfg(test)]
mod tests {
    use super::Percent;
    const EIGHTEEN: Percent = Percent(18);
    const FIFTY: usize = 50;
    const NINE: usize = 9;

    #[test]
    fn a_percent_of_a_whole_rounds_down() {
        assert_eq!(EIGHTEEN.applied_to::<usize, usize>(FIFTY), NINE);
    }
}
