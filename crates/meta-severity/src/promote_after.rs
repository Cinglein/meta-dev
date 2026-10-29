use std::num::NonZeroUsize;

use meta_schema::LevelName;
use serde::{Deserialize, Serialize};

use crate::evidence::Evidence;
use crate::rating_failure::RatingFailure;
use crate::scale::Scale;
use crate::signal::Signal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromoteAfter(NonZeroUsize);

impl PromoteAfter {
    pub fn promote<About: PartialEq, Since>(
        self,
        scale: &Scale,
        history: &[Signal<About, Since>],
    ) -> Result<Option<LevelName>, RatingFailure> {
        history.last().map_or(Ok(None), |latest| {
            let same: Vec<&Signal<About, Since>> =
                history.iter().filter(|signal| signal.identity() == latest.identity()).collect();
            let earlier = same.iter().rev().nth(1).and_then(|signal| signal.level());
            scale.moved(earlier, latest.level(), latest.evidence())?;
            Ok(latest.level().map(|level| self.escalated(scale, &same, level)))
        })
    }

    fn escalated<About, Since>(self, scale: &Scale, same: &[&Signal<About, Since>], level: &LevelName) -> LevelName {
        let run = same.iter().rev().take_while(|signal| signal.level() == Some(level)).count();
        let promoted = (run >= self.0.get()
            || same.last().is_some_and(|signal| *signal.evidence() == Evidence::FailedRemedy))
        .then(|| scale.worse(level))
        .flatten();
        promoted.unwrap_or(level).clone()
    }
}
