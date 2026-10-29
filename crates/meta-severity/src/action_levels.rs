use std::collections::BTreeMap;

use meta_schema::{Division, LevelName, TopDivision};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::promoted::Promoted;
use crate::rating_failure::RatingFailure;
use crate::scales::Scales;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionLevels(BTreeMap<Division, LevelName>);

impl ActionLevels {
    #[must_use]
    pub fn acts(&self, scales: &Scales, division: &Division, promoted: &Promoted) -> bool {
        let line = division.lineage().into_iter().rev().find_map(|at| self.0.get(&at).map(|line| (at, line)));
        line.is_none_or(|(at, line)| {
            let rolled = promoted.reading().rolled_up(scales, division, &at);
            scales.scale(&at).is_none_or(|scale| scale.reaches(&rolled, line))
        })
    }

    pub fn leveled(&self, scales: &Scales, domained: impl Fn(&Division) -> bool) -> Result<(), RatingFailure> {
        let unknown = self.0.keys().find(|at| !scales.declares(at) && !domained(at));
        unknown.map_or(Ok(()), |at| Err(RatingFailure::NoSuchDivision(at.clone())))?;
        let unacted = TopDivision::iter().map(Division::from).find(|top| !self.0.contains_key(top));
        unacted.map_or(Ok(()), |top| Err(RatingFailure::Unacted(top)))?;
        let unknown =
            self.0.iter().find(|(at, line)| scales.scale(at).and_then(|scale| scale.index_of(line)).is_none());
        unknown.map_or(Ok(()), |(_, line)| Err(RatingFailure::UnknownLevel(line.clone())))
    }
}
