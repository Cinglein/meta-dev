use std::cmp::Ordering;
use std::collections::BTreeMap;

use meta_schema::{LevelName, Rating};
use serde::{Deserialize, Serialize};

use crate::level::Level;
use crate::rating_failure::RatingFailure;
use crate::reading::Reading;
use crate::scale::Scale;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Thresholds<Measure>(BTreeMap<LevelName, Measure>);

impl<Measure: PartialOrd> Thresholds<Measure> {
    pub fn rate(&self, scale: &Scale, measure: Option<&Measure>) -> Result<Reading, RatingFailure> {
        let unknown = self.0.keys().find(|level| scale.index_of(level).is_none());
        unknown.map_or(Ok(()), |level| Err(RatingFailure::UnknownLevel(level.clone())))?;
        let lined: Vec<&LevelName> =
            scale.levels().iter().map(Level::name).filter(|level| self.0.contains_key(*level)).collect();
        let disordered = lined.windows(Scale::PAIR_LEN).find_map(|pair| match pair {
            [worse, better] if self.0.get(*worse).partial_cmp(&self.0.get(*better)) != Some(Ordering::Greater) => {
                Some(*worse)
            }
            _ => None,
        });
        disordered.map_or(Ok(()), |level| Err(RatingFailure::Disordered(level.clone())))?;
        Ok(measure.map_or(Reading::from(Rating::Unrated), |measure| self.over(&lined, measure)))
    }

    fn over(&self, lined: &[&LevelName], measure: &Measure) -> Reading {
        let over = lined.iter().find(|level| self.0.get(**level).is_some_and(|line| measure > line));
        over.map_or(Reading::Clear, |level| Reading::from(Rating::Rated((*level).clone())))
    }
}

impl<Measure> From<BTreeMap<LevelName, Measure>> for Thresholds<Measure> {
    fn from(lines: BTreeMap<LevelName, Measure>) -> Thresholds<Measure> {
        Thresholds(lines)
    }
}
