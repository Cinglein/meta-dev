use meta_schema::{Division, LevelName};
use serde::{Deserialize, Serialize};

use crate::level::Level;
use crate::scale_failure::ScaleFailure;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Scale(Vec<Level>);

impl Scale {
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

    pub(crate) fn mapped_onto(&self, division: &Division, parent: Option<&Scale>) -> Result<(), ScaleFailure> {
        let refused = |failure: fn(Division, LevelName) -> ScaleFailure, level: &Level| {
            failure(division.clone(), level.name().clone())
        };
        let rank = |level: &Level| match (parent, level.parent()) {
            (None, None) => Ok(None),
            (None, Some(_)) => Err(refused(ScaleFailure::TopMapped, level)),
            (Some(parent), named) => named
                .and_then(|name| parent.0.iter().position(|above| above.name() == name))
                .map(Some)
                .ok_or_else(|| refused(ScaleFailure::Unmapped, level)),
        };
        let ranks = self.0.iter().map(rank).collect::<Result<Vec<_>, ScaleFailure>>()?;
        let twice = self
            .0
            .iter()
            .enumerate()
            .find(|(at, level)| self.0.iter().take(*at).any(|before| before.name() == level.name()));
        let broken = ranks
            .windows(2)
            .zip(self.0.iter().skip(1))
            .find(|(pair, _)| matches!(pair, [Some(above), Some(below)] if below < above));
        match (self.0.first(), twice, broken) {
            (None, ..) => Err(ScaleFailure::Empty(division.clone())),
            (Some(_), Some((_, level)), _) => Err(refused(ScaleFailure::Twice, level)),
            (Some(_), None, Some((_, level))) => Err(refused(ScaleFailure::OrderBroken, level)),
            (Some(_), None, None) => Ok(()),
        }
    }
}
