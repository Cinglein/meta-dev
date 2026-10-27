use std::fmt;

use meta_schema::{Division, LevelName, Rating, Target};
use meta_state::{RatedItem, State};
use serde::{Deserialize, Serialize};

use crate::scales::Scales;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityFilter {
    Unrated,
    AtLevel(LevelName),
}

impl SeverityFilter {
    #[must_use]
    pub const fn level(&self) -> Option<&LevelName> {
        match self {
            Self::Unrated => None,
            Self::AtLevel(level) => Some(level),
        }
    }

    #[must_use]
    pub fn admitted(&self, scales: &Scales, division: &Division, item: &RatedItem) -> Option<Target> {
        let inside = |placed: &&Division| *placed == division || division.strictly_contains(placed);
        let admitted = match (item.placement().division().filter(inside), self, item.rating().level()) {
            (Some(_), Self::Unrated, None) => Some(()),
            (Some(placed), Self::AtLevel(level), Some(rated)) => {
                scales.rolled_up(placed, rated, division).filter(|rolled| rolled == level).map(|_| ())
            }
            _ => None,
        };
        admitted.map(|()| item.target())
    }

    #[must_use]
    pub fn covers(&self, item: &RatedItem, scales: &Scales, division: &Division) -> Option<Target> {
        let inside = |placed: &&Division| *placed == division || division.strictly_contains(placed);
        let rolled = item
            .placement()
            .division()
            .filter(inside)
            .zip(item.rating().level())
            .and_then(|(placed, rated)| scales.rolled_up(placed, rated, division));
        let ranks = scales.scale(division).zip(rolled).zip(self.level());
        let at_or_above = ranks.and_then(|((scale, rolled), level)| scale.index_of(&rolled).zip(scale.index_of(level)));
        let covered = at_or_above.filter(|(found, capped)| found <= capped).map(|_| item.target());
        covered.or_else(|| self.admitted(scales, division, item))
    }

    #[must_use]
    pub fn open(&self, scales: &Scales, division: &Division, state: &State) -> Vec<Target> {
        state.open_items().iter().filter_map(|item| self.admitted(scales, division, item)).collect()
    }
}

impl fmt::Display for SeverityFilter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unrated => Rating::Unrated.fmt(formatter),
            Self::AtLevel(level) => level.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests;
