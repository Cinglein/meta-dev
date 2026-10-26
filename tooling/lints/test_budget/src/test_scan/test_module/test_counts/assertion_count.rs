use derive_more::{AddAssign, Display, From};

use crate::measurement::Measurement;
use task::{Limit, Policy};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, AddAssign, Display, From)]
pub struct AssertionCount(usize);

impl AssertionCount {
    pub(super) fn measured(self, policy: &Policy) -> Measurement<AssertionCount> {
        Measurement::new(
            "assertions",
            self,
            policy.limit(Limit::AssertionsPerTest).counted(),
        )
    }
}
