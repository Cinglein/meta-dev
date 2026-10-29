use std::cmp::Ordering;
use std::num::NonZeroUsize;

use meta_schema::Rating;
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
    const OWNER_SET_RUN_LENGTH: usize = 3;

    pub fn promote<About: PartialEq, Since>(
        self,
        scale: &Scale,
        history: &[Signal<About, Since>],
    ) -> Result<Promoted, RatingFailure> {
        history.last().map_or(Ok(Promoted::new(Reading::from(Rating::Unrated))), |latest| {
            let known: Vec<&Reading> = history
                .iter()
                .filter(|signal| signal.identity() == latest.identity())
                .filter_map(|signal| signal.reading().known())
                .collect();
            let earlier = known.iter().rev().nth(1);
            let rise = earlier
                .map_or(Ok(Ordering::Equal), |earlier| scale.moved(earlier, latest.reading(), latest.evidence()))?;
            let run = known.iter().rev().take_while(|reading| **reading == latest.reading()).count();
            let stepped = (run >= self.0.get()
                || matches!((latest.evidence(), rise), (Evidence::FailedRemedy, Ordering::Equal | Ordering::Greater)))
            .then(|| scale.worse(latest.reading()))
            .flatten();
            Ok(Promoted::new(stepped.unwrap_or_else(|| latest.reading().clone())))
        })
    }
}

impl Default for PromoteAfter {
    fn default() -> PromoteAfter {
        PromoteAfter(NonZeroUsize::new(Self::OWNER_SET_RUN_LENGTH).unwrap_or(NonZeroUsize::MIN))
    }
}
