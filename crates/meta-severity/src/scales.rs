use std::collections::BTreeMap;

use meta_schema::{Division, LevelName, TopDivision};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::level::Level;
use crate::scale::Scale;
use crate::scale_failure::ScaleFailure;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "BTreeMap<Division, Scale>", into = "BTreeMap<Division, Scale>")]
pub struct Scales(BTreeMap<Division, Scale>);

impl Scales {
    const DEFAULT_LEVELS: [(&str, &str); 3] = [
        ("critical", "breaks the property: crash, blocked use or a wrong result; needs a reproduction"),
        ("major", "degrades the property; needs a concrete failure scenario"),
        ("minor", "a nuisance; the property holds"),
    ];

    #[must_use]
    pub fn owner(&self, division: &Division) -> Option<&Division> {
        division.lineage().iter().rev().find_map(|owner| self.0.get_key_value(owner)).map(|(owner, _)| owner)
    }

    #[must_use]
    pub fn scale(&self, division: &Division) -> Option<&Scale> {
        self.owner(division).and_then(|owner| self.0.get(owner))
    }

    #[must_use]
    pub fn rolled_up(&self, from: &Division, level: &LevelName, onto: &Division) -> Option<LevelName> {
        let owner = self.owner(from)?;
        let target = self.owner(onto)?;
        let mapped = self.scale(from)?.level(level)?;
        match mapped.parent() {
            _ if owner == target => Some(level.clone()),
            Some(parent) if target.strictly_contains(owner) => {
                self.rolled_up(owner.lineage().iter().rev().nth(1)?, parent, onto)
            }
            Some(_) | None => None,
        }
    }

    fn placed(&self, division: &Division, scale: &Scale) -> Result<(), ScaleFailure> {
        let lineage = division.lineage();
        let parent = lineage.iter().rev().nth(1).and_then(|parent| self.scale(parent));
        scale.mapped_onto(division, parent)
    }
}

impl Default for Scales {
    fn default() -> Scales {
        let level = |(name, meaning): (&str, &str)| {
            name.parse().ok().zip(meaning.parse().ok()).map(|(name, meaning)| Level::top(name, meaning))
        };
        let scale = Scale::new(Self::DEFAULT_LEVELS.into_iter().filter_map(level).collect());
        Scales(TopDivision::iter().map(|top| (Division::from(top), scale.clone())).collect())
    }
}

impl TryFrom<BTreeMap<Division, Scale>> for Scales {
    type Error = ScaleFailure;

    fn try_from(declared: BTreeMap<Division, Scale>) -> Result<Scales, ScaleFailure> {
        let mut scales = Scales::default();
        scales.0.extend(declared);
        scales.0.iter().try_for_each(|(division, scale)| scales.placed(division, scale))?;
        Ok(scales)
    }
}

impl From<Scales> for BTreeMap<Division, Scale> {
    fn from(scales: Scales) -> BTreeMap<Division, Scale> {
        scales.0
    }
}

#[cfg(test)]
mod tests;
