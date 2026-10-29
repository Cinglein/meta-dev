use meta_schema::LevelName;

use crate::rating_failure::RatingFailure;
use crate::scale::Scale;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thresholds<Measure>(Vec<(Measure, LevelName)>);

impl<Measure: PartialOrd> Thresholds<Measure> {
    pub fn on(scale: &Scale, mut lines: Vec<(Measure, LevelName)>) -> Result<Thresholds<Measure>, RatingFailure> {
        let unknown = lines.iter().find(|(_, level)| scale.index_of(level).is_none());
        unknown.map_or(Ok(()), |(_, level)| Err(RatingFailure::UnknownLevel(level.clone())))?;
        lines.sort_by_key(|(_, level)| scale.index_of(level));
        Ok(Thresholds(lines))
    }

    #[must_use]
    pub fn rate(&self, measure: &Measure) -> Option<&LevelName> {
        self.0.iter().find(|(line, _)| measure > line).map(|(_, level)| level)
    }
}
