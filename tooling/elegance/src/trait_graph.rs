mod depths;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::facts::Facts;
use crate::tally::Tally;
use crate::trait_key::TraitKey;
use depths::Depths;

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
    pub fn depth(&self) -> Tally {
        let keys: BTreeSet<&TraitKey> = self.supertraits.keys().chain(self.blankets.keys()).collect();
        let depths = keys.iter().fold(Depths::default(), |depths, key| self.settled(key, &BTreeSet::new(), depths));
        keys.iter().filter_map(|key| depths.settled_depth(key)).max().unwrap_or_default()
    }

    fn settled<'graph>(
        &'graph self,
        key: &'graph TraitKey,
        path: &BTreeSet<&'graph TraitKey>,
        depths: Depths<'graph>,
    ) -> Depths<'graph> {
        if depths.settled_depth(key).is_some() {
            depths
        } else {
            let below: BTreeSet<&TraitKey> = path.iter().copied().chain(std::iter::once(key)).collect();
            let bases: Vec<&TraitKey> =
                self.rests_on(key).into_iter().filter(|base| !below.contains(base)).collect();
            let depths = bases.iter().fold(depths, |depths, base| self.settled(base, &below, depths));
            let deepest =
                bases.iter().filter_map(|base| depths.settled_depth(base)).map(|depth| Tally::ONE + depth).max();
            depths.settle(key, deepest.unwrap_or_default())
        }
    }
}

impl fmt::Display for TraitGraph {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "graph edges: supertrait {}, blanket {}; depth {}",
            self.supertraits.values().map(Tally::counting).sum::<Tally>(),
            self.blankets.values().map(Tally::counting).sum::<Tally>(),
            self.depth()
        )
    }
}
