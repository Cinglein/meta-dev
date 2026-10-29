use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use task::Percent;

use crate::effect::Effect;
use crate::item_count::ItemCount;

#[derive(Clone, Debug, Default)]
pub struct EffectTally {
    units: ItemCount,
    touching: ItemCount,
    families: BTreeMap<Effect, ItemCount>,
}

impl EffectTally {
    pub fn of_units(units: &[BTreeSet<Effect>]) -> EffectTally {
        let families = units
            .iter()
            .flatten()
            .fold(BTreeMap::new(), |mut families, effect| {
                let count: &mut ItemCount = families.entry(*effect).or_default();
                *count = *count + ItemCount::of_len(1);
                families
            });
        EffectTally {
            units: ItemCount::of_len(units.len()),
            touching: ItemCount::of_len(units.iter().filter(|effects| !effects.is_empty()).count()),
            families,
        }
    }

    pub fn share(&self) -> Percent {
        Percent::rounded_up(self.touching.count(), self.units.count())
    }

    pub fn dispersion(&self) -> ItemCount {
        self.families.values().max().copied().unwrap_or_default()
    }

    pub fn touched(&self) -> Option<&EffectTally> {
        (self.touching > ItemCount::default()).then_some(self)
    }
}

impl fmt::Display for EffectTally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let families: Vec<String> = self
            .families
            .iter()
            .map(|(effect, units)| format!("{effect} {units}"))
            .collect();
        write!(
            formatter,
            "{} of {} units, share {}%: {}",
            self.touching,
            self.units,
            self.share(),
            families.join(", ")
        )
    }
}
