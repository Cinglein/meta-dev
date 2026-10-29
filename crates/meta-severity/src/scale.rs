use std::cmp::Ordering;

use meta_schema::{Change, Division, LevelName, Rating, Scenario};
use serde::{Deserialize, Serialize};

use crate::evidence::Evidence;
use crate::level::Level;
use crate::rating_failure::RatingFailure;
use crate::reading::Reading;
use crate::scale_failure::ScaleFailure;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Scale(Vec<Level>);

impl Scale {
    pub(crate) const PAIR_LEN: usize = 2;
    const DEFAULT_LEVELS: &str = include_str!("default_levels.json");

    #[must_use]
    pub fn new(levels: Vec<Level>) -> Scale {
        Scale(levels)
    }

    #[must_use]
    pub fn top_default() -> Scale {
        serde_json::from_str(Self::DEFAULT_LEVELS).unwrap_or_else(|_| Scale(Vec::new()))
    }

    #[must_use]
    pub fn levels(&self) -> &[Level] {
        &self.0
    }

    #[must_use]
    pub fn level(&self, name: &LevelName) -> Option<&Level> {
        self.0.iter().find(|level| level.name() == name)
    }

    #[must_use]
    pub fn index_of(&self, name: &LevelName) -> Option<usize> {
        self.0.iter().position(|level| level.name() == name)
    }

    #[must_use]
    pub fn same_levels(&self, other: &Scale) -> bool {
        self.0.iter().map(Level::name).eq(other.0.iter().map(Level::name))
    }

    pub fn change(
        &self,
        current: &Rating,
        level: LevelName,
        scenario: Option<Scenario>,
    ) -> Result<Change, RatingFailure> {
        let to = Reading::from(Rating::Rated(level.clone()));
        let rise = self.moved(&Reading::from(current.clone()), &to, &Evidence::from(scenario.clone()))?;
        match (current.level().cloned(), rise, scenario) {
            (None, ..) => Ok(Change::First { level }),
            (Some(_), Ordering::Equal, _) => Err(RatingFailure::Unchanged(level)),
            (Some(from), Ordering::Greater, _) => Ok(Change::Lowered { from, level }),
            (Some(from), Ordering::Less, scenario) => scenario
                .map(|scenario| Change::Raised { from: from.clone(), level, scenario })
                .ok_or_else(|| RatingFailure::RaisedWithoutEvidence(Reading::from(Rating::Rated(from)))),
        }
    }

    pub fn moved(&self, from: &Reading, to: &Reading, evidence: &Evidence) -> Result<Ordering, RatingFailure> {
        let new = to.index_on(self)?;
        let rise = match from.index_on(self) {
            Err(_) => new.map_or(Ordering::Equal, |_| Ordering::Less),
            Ok(old) => new.zip(old).map_or(Ordering::Equal, |(new, old)| new.cmp(&old)),
        };
        evidence.admits(from, rise)
    }

    #[must_use]
    pub fn worse(&self, reading: &Reading) -> Option<Reading> {
        let above = reading.index_on(self).ok().flatten()?.checked_sub(1)?;
        self.0.get(above).map(|level| Reading::from(Rating::Rated(level.name().clone())))
    }

    #[must_use]
    pub fn reaches(&self, reading: &Reading, line: &LevelName) -> bool {
        let found = reading.index_on(self).ok().flatten();
        found.zip(self.index_of(line)).is_none_or(|(found, line)| found <= line)
    }

    pub fn known_index(&self, level: &LevelName) -> Result<usize, RatingFailure> {
        self.index_of(level).ok_or_else(|| RatingFailure::UnknownLevel(level.clone()))
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
        let uncovered = parent.and_then(|parent| parent.uncovered_by(self));
        let renamed = parent.and_then(|parent| parent.renamed_in(self));
        let empty = self.0.first().map_or(Some(ScaleFailure::Empty(division.clone())), |_| None);
        let failure = empty
            .or_else(|| top_mapped.map(|level| refused(ScaleFailure::TopMapped, level)))
            .or_else(|| unmapped.map(|(level, _)| refused(ScaleFailure::Unmapped, level)))
            .or_else(|| renamed.map(|level| refused(ScaleFailure::Renamed, level)))
            .or_else(|| twice.map(|(_, level)| refused(ScaleFailure::Twice, level)))
            .or_else(|| broken.map(|(_, level)| refused(ScaleFailure::OrderBroken, level)))
            .or_else(|| uncovered.map(|above| refused(ScaleFailure::Uncovered, above)));
        failure.map_or(Ok(()), Err)
    }

    fn renamed_in<'child>(&self, child: &'child Scale) -> Option<&'child Level> {
        child.0.iter().find(|level| self.level(level.name()).is_some() && level.parent() != Some(level.name()))
    }

    fn uncovered_by(&self, child: &Scale) -> Option<&Level> {
        self.0.iter().find(|above| child.0.iter().all(|level| level.parent() != Some(above.name())))
    }
}

#[cfg(test)]
mod tests;
