use std::cmp::Ordering;

use meta_schema::{Change, Division, LevelName, Rating, Scenario};
use serde::{Deserialize, Serialize};

use crate::evidence::Evidence;
use crate::level::Level;
use crate::rating_failure::RatingFailure;
use crate::scale_failure::ScaleFailure;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Scale(Vec<Level>);

impl Scale {
    const PAIR_LEN: usize = 2;
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
        let rank = |name: &LevelName| self.0.iter().position(|known| known.name() == name);
        let new = rank(&level).ok_or_else(|| RatingFailure::UnknownLevel(level.clone()))?;
        match current.level().map(|from| (from.clone(), rank(from).map(|old| new.cmp(&old)))) {
            None => Ok(Change::First { level }),
            Some((_, Some(Ordering::Equal))) => Err(RatingFailure::Unchanged(level)),
            Some((from, Some(Ordering::Greater))) => Ok(Change::Lowered { from, level }),
            Some((from, Some(Ordering::Less) | None)) => scenario
                .map(|scenario| Change::Raised { from: from.clone(), level, scenario })
                .ok_or(RatingFailure::RaisedWithoutScenario(from)),
        }
    }

    pub fn moved(
        &self,
        from: Option<&LevelName>,
        to: Option<&LevelName>,
        evidence: &Evidence,
    ) -> Result<(), RatingFailure> {
        let rank = |level: &LevelName| self.index_of(level).ok_or_else(|| RatingFailure::UnknownLevel(level.clone()));
        let ranks = from.zip(to).map(|(from, to)| Ok::<_, RatingFailure>((from, rank(from)?, rank(to)?)));
        match ranks.transpose()? {
            Some((from, old, new)) if new > old && !evidence.supports_raise() => {
                Err(RatingFailure::RaisedWithoutEvidence(from.clone()))
            }
            _ => Ok(()),
        }
    }

    #[must_use]
    pub fn worse(&self, level: &LevelName) -> Option<&LevelName> {
        let above = self.index_of(level)?.checked_sub(1)?;
        self.0.get(above).map(Level::name)
    }

    #[must_use]
    pub fn reaches(&self, level: Option<&LevelName>, line: &LevelName) -> bool {
        let ranks = level.and_then(|level| self.index_of(level)).zip(self.index_of(line));
        ranks.is_some_and(|(found, line)| found >= line)
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
