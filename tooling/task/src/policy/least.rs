use std::num::NonZeroU64;

use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

use crate::budget::Millis;
use crate::percent::Percent;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, From, Into, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Least(usize);

impl Least {
    pub const fn allows(self, length: usize) -> bool {
        length >= self.0
    }

    #[must_use]
    pub fn percent_above(self, price: Millis) -> Millis {
        price.plus(Percent::from(self.0).portion_of(price))
    }

    #[must_use]
    pub fn times_rounded_up(self, each: Millis, step: NonZeroU64) -> Millis {
        each.times(self.0).rounded_up_to(step)
    }
}
