use std::collections::{BTreeMap, BTreeSet};

use crate::facts::Facts;
use crate::impl_fact::ImplFact;
use crate::signature::Signature;
use crate::tally::Tally;
use crate::trait_key::TraitKey;
use crate::trait_shape::TraitShape;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitCatalog(BTreeMap<TraitKey, BTreeSet<Signature>>);

impl TraitCatalog {
    #[must_use]
    pub fn cataloged(facts: &Facts) -> TraitCatalog {
        TraitCatalog(
            facts
                .traits()
                .keys()
                .chain(facts.impls().iter().filter_map(|fact| fact.kind().implemented()))
                .map(|key| (key.clone(), Self::methods_declared_or_written(facts, key)))
                .collect(),
        )
    }

    #[must_use]
    pub fn methods_declared_or_written(facts: &Facts, key: &TraitKey) -> BTreeSet<Signature> {
        facts
            .traits()
            .get(key)
            .into_iter()
            .flat_map(TraitShape::methods)
            .chain(facts.impls_of(key).flat_map(ImplFact::bodies))
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn methods(&self, key: &TraitKey) -> BTreeSet<&Signature> {
        self.0.get(key).into_iter().flatten().collect()
    }

    #[must_use]
    pub fn usable<'keys>(&self, keys: impl IntoIterator<Item = &'keys TraitKey>) -> Tally {
        keys.into_iter().map(|key| Tally::counting(self.methods(key))).sum()
    }
}
