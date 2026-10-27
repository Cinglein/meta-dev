use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::division::Division;
use crate::level::Level;
use crate::level_name::LevelName;
use crate::scale::Scale;
use crate::scale_failure::ScaleFailure;
use crate::top_division::TopDivision;

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
    pub fn effective(&self, division: &Division) -> Option<(&Division, &Scale)> {
        division.lineage().iter().rev().find_map(|owner| self.0.get_key_value(owner))
    }

    #[must_use]
    pub fn rolled_up(&self, from: &Division, level: &LevelName, onto: &Division) -> Option<LevelName> {
        let (owner, scale) = self.effective(from)?;
        let (target, _) = self.effective(onto)?;
        let mapped = scale.level(level)?;
        match mapped.parent() {
            _ if owner == target => Some(level.clone()),
            Some(parent) if target.strictly_contains(owner) => {
                let lineage = owner.lineage();
                self.rolled_up(lineage.iter().rev().nth(1)?, parent, onto)
            }
            Some(_) | None => None,
        }
    }

    fn placed(&self, division: &Division, scale: &Scale) -> Result<(), ScaleFailure> {
        let lineage = division.lineage();
        let parent = lineage.iter().rev().nth(1).and_then(|parent| self.effective(parent));
        scale.mapped_onto(division, parent.map(|(_, scale)| scale))
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
        let _unchecked = scales.0.iter().try_for_each(|(division, scale)| scales.placed(division, scale));
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
