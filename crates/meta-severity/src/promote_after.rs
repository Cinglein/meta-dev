use std::cmp::Ordering;
use std::iter;
use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};

use crate::evidence::Evidence;
use crate::promoted::Promoted;
use crate::rating_failure::RatingFailure;
use crate::reading::Reading;
use crate::scale::Scale;
use crate::signal::Signal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromoteAfter(NonZeroUsize);

impl PromoteAfter {
    pub(crate) fn promote<About: PartialEq, Since>(
        self,
        scale: &Scale,
        latest: &Signal<About, Since>,
        earlier: &[Signal<About, Since>],
    ) -> Result<Promoted, RatingFailure> {
        let known: Vec<&Reading> = earlier
            .iter()
            .filter(|signal| signal.identity() == latest.identity())
            .chain(iter::once(latest))
            .filter_map(|signal| signal.reading().known())
            .collect();
        let before = known.iter().rev().nth(1);
        let rise =
            before.map_or(Ok(Ordering::Equal), |before| scale.moved(before, latest.reading(), latest.evidence()))?;
        let run = known.iter().rev().take_while(|reading| **reading == latest.reading()).count();
        let stepped = (run >= self.0.get()
            || matches!((latest.evidence(), rise), (Evidence::FailedRemedy, Ordering::Equal | Ordering::Greater)))
        .then(|| scale.worse(latest.reading()))
        .flatten();
        Ok(Promoted::new(stepped.unwrap_or_else(|| latest.reading().clone())))
    }
}
