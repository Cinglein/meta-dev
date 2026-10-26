use std::num::NonZeroUsize;

use task::Millis;
use task::MutantCount;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    count: MutantCount,
    millis: Millis,
}

impl Tally {
    #[must_use]
    pub fn one(millis: Millis) -> Tally {
        Tally { count: MutantCount::from(1), millis }
    }

    #[must_use]
    pub fn plus(self, other: Tally) -> Tally {
        Tally { count: self.count + other.count, millis: self.millis.plus(other.millis) }
    }

    #[must_use]
    pub fn mean(self) -> Millis {
        NonZeroUsize::new(self.count.into()).map_or(Millis::default(), |count| self.millis.divided_by(count))
    }
}
