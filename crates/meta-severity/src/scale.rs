use meta_schema::{Change, Division, LevelName, Rating, Scenario};
use serde::{Deserialize, Serialize};

use crate::level::Level;
use crate::rating_failure::RatingFailure;
use crate::scale_failure::ScaleFailure;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Scale(Vec<Level>);

impl Scale {
    const PAIR_LEN: usize = 2;

    #[must_use]
    pub fn new(levels: Vec<Level>) -> Scale {
        Scale(levels)
    }

    #[must_use]
    pub fn levels(&self) -> &[Level] {
        &self.0
    }

    #[must_use]
    pub fn level(&self, name: &LevelName) -> Option<&Level> {
        self.0.iter().find(|level| level.name() == name)
    }

    pub fn change(
        &self,
        current: &Rating,
        level: LevelName,
        scenario: Option<Scenario>,
    ) -> Result<Change, RatingFailure> {
        let rank = |name: &LevelName| self.0.iter().position(|known| known.name() == name);
        let new = rank(&level).ok_or_else(|| RatingFailure::UnknownLevel(level.clone()))?;
        match current.level() {
            None => Ok(Change::First { level }),
            Some(from) if *from == level => Err(RatingFailure::Unchanged(level)),
            Some(from) if rank(from).is_some_and(|old| new > old) => Ok(Change::Lowered { from: from.clone(), level }),
            Some(from) => scenario
                .map(|scenario| Change::Raised { from: from.clone(), level, scenario })
                .ok_or_else(|| RatingFailure::RaisedWithoutScenario(from.clone())),
        }
    }

    fn parent_index(&self, level: &Level) -> Option<usize> {
        level.parent().and_then(|name| self.0.iter().position(|above| above.name() == name))
    }

    pub(crate) fn mapped_onto(&self, division: &Division, parent: Option<&Scale>) -> Result<(), ScaleFailure> {
        let refused = |failure: fn(Division, LevelName) -> ScaleFailure, level: &Level| {
            failure(division.clone(), level.name().clone())
        };
        let ranks: Vec<Option<usize>> =
            self.0.iter().map(|level| parent.and_then(|parent| parent.parent_index(level))).collect();
        let top_mapped = self.0.iter().find(|level| parent.is_none() && level.parent().is_some());
        let unmapped = self.0.iter().zip(&ranks).find(|(_, rank)| parent.is_some() && rank.is_none());
        let twice = self
            .0
            .iter()
            .enumerate()
            .find(|(at, level)| self.0.iter().take(*at).any(|before| before.name() == level.name()));
        let broken = ranks
            .windows(Self::PAIR_LEN)
            .zip(self.0.iter().skip(1))
            .find(|(pair, _)| matches!(pair, [Some(above), Some(below)] if below < above));
        match (self.0.first(), top_mapped, unmapped, twice, broken) {
            (None, ..) => Err(ScaleFailure::Empty(division.clone())),
            (Some(_), Some(level), ..) => Err(refused(ScaleFailure::TopMapped, level)),
            (Some(_), None, Some((level, _)), ..) => Err(refused(ScaleFailure::Unmapped, level)),
            (Some(_), None, None, Some((_, level)), _) => Err(refused(ScaleFailure::Twice, level)),
            (Some(_), None, None, None, Some((_, level))) => Err(refused(ScaleFailure::OrderBroken, level)),
            (Some(_), None, None, None, None) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests;
