use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::facts::Facts;
use crate::tally::Tally;
use crate::trait_key::TraitKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitGraph {
    supertraits: BTreeMap<TraitKey, BTreeSet<TraitKey>>,
    blankets: BTreeMap<TraitKey, BTreeSet<TraitKey>>,
}

impl TraitGraph {
    #[must_use]
    pub fn drawn(facts: &Facts) -> TraitGraph {
        TraitGraph {
            supertraits: facts.traits().iter().map(|(key, shape)| (key.clone(), shape.supertraits().clone())).collect(),
            blankets: facts
                .impls()
                .iter()
                .filter_map(|fact| fact.kind().implemented().zip(fact.kind().blanket_bounds()))
                .fold(BTreeMap::new(), |mut blankets, (implemented, bounds)| {
                    blankets.entry(implemented.clone()).or_insert_with(BTreeSet::new).extend(bounds.iter().cloned());
                    blankets
                }),
        }
    }

    #[must_use]
    pub fn rests_on(&self, key: &TraitKey) -> BTreeSet<&TraitKey> {
        self.supertraits.get(key).into_iter().chain(self.blankets.get(key)).flatten().collect()
    }

    #[must_use]
    pub fn linked(&self, keys: &BTreeSet<TraitKey>) -> bool {
        keys.iter().flat_map(|key| self.rests_on(key)).any(|base| keys.contains(base))
    }

    #[must_use]
    pub fn depth_from(&self, key: &TraitKey, visiting: &BTreeSet<&TraitKey>) -> Tally {
        let path: BTreeSet<&TraitKey> = visiting.iter().copied().chain(std::iter::once(key)).collect();
        self.rests_on(key)
            .into_iter()
            .filter(|base| !path.contains(base))
            .map(|base| Tally::ONE + self.depth_from(base, &path))
            .max()
            .unwrap_or_default()
    }
}

impl fmt::Display for TraitGraph {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let depth = self
            .supertraits
            .keys()
            .chain(self.blankets.keys())
            .map(|key| self.depth_from(key, &BTreeSet::new()))
            .max()
            .unwrap_or_default();
        write!(
            formatter,
            "graph edges: supertrait {}, blanket {}; depth {depth}",
            self.supertraits.values().map(Tally::counting).sum::<Tally>(),
            self.blankets.values().map(Tally::counting).sum::<Tally>()
        )
    }
}
