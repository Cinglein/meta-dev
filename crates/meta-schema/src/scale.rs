use serde::{Deserialize, Serialize};

use crate::division::Division;
use crate::level::Level;
use crate::level_name::LevelName;
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
        let failed = |level: &Level, failure: fn(Division, LevelName) -> ScaleFailure| {
            Err(failure(division.clone(), level.name().clone()))
        };
        let rank = |level: &Level| {
            let parent = parent?;
            level.parent().and_then(|name| parent.0.iter().position(|above| above.name() == name))
        };
        self.0.first().map_or(Err(ScaleFailure::Empty(division.clone())), |_| Ok(()))?;
        let mut floor = None;
        self.0.iter().enumerate().try_for_each(|(at, level)| {
            let twice = self.0.iter().take(at).any(|before| before.name() == level.name());
            match (twice, parent, level.parent(), rank(level)) {
                (true, ..) => failed(level, ScaleFailure::Twice),
                (false, None, Some(_), _) => failed(level, ScaleFailure::TopMapped),
                (false, Some(_), _, None) => failed(level, ScaleFailure::Unmapped),
                (false, _, _, Some(ranked)) if floor.is_some_and(|floor| ranked < floor) => {
                    failed(level, ScaleFailure::OrderBroken)
                }
                (false, ..) => {
                    floor = rank(level);
                    Ok(())
                }
            }
        })
    }
}
