use std::collections::BTreeMap;

use meta_schema::LevelName;
use serde::{Deserialize, Serialize};

use crate::level::Level;
use crate::rating_failure::RatingFailure;
use crate::scale::Scale;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Thresholds<Measure>(BTreeMap<LevelName, Measure>);

impl<Measure: PartialOrd> Thresholds<Measure> {
    pub fn rate(&self, scale: &Scale, measure: &Measure) -> Result<Option<LevelName>, RatingFailure> {
        let unknown = self.0.keys().find(|level| scale.index_of(level).is_none());
        unknown.map_or(Ok(()), |level| Err(RatingFailure::UnknownLevel(level.clone())))?;
        let over =
            scale.levels().iter().map(Level::name).find(|level| self.0.get(level).is_some_and(|line| measure > line));
        Ok(over.cloned())
    }
}

impl<Measure> From<BTreeMap<LevelName, Measure>> for Thresholds<Measure> {
    fn from(lines: BTreeMap<LevelName, Measure>) -> Thresholds<Measure> {
        Thresholds(lines)
    }
}
