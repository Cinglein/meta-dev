use derive_more::{AddAssign, Display, From};

use crate::measurement::Measurement;
use task::{Limit, Policy};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, AddAssign, Display, From)]
pub struct LiteralCount(usize);

impl LiteralCount {
    pub(super) fn measured(self, policy: &Policy) -> Measurement<LiteralCount> {
        Measurement::new(
            "literals",
            self,
            policy.limit(Limit::LiteralsPerTest).counted(),
        )
    }
}
