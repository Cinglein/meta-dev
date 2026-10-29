use std::collections::{BTreeMap, BTreeSet};

use crate::facts::Facts;
use crate::impl_fact::ImplFact;
use crate::method_name::MethodName;
use crate::signature::Signature;
use crate::tally::Tally;
use crate::trait_key::TraitKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitCatalog(BTreeMap<TraitKey, BTreeSet<MethodName>>);

impl TraitCatalog {
    #[must_use]
    pub fn cataloged(facts: &Facts) -> TraitCatalog {
        TraitCatalog(
            facts
                .traits()
                .keys()
                .chain(facts.impls().iter().filter_map(|fact| fact.kind().implemented()))
                .map(|key| (key.clone(), Self::methods_named(facts, key)))
                .collect(),
        )
    }

    #[must_use]
    pub fn methods_named(facts: &Facts, key: &TraitKey) -> BTreeSet<MethodName> {
        match facts.traits().get(key) {
            Some(shape) => shape.methods().iter().map(Signature::name).cloned().collect(),
            None => facts.impls_of(key).flat_map(ImplFact::bodies).map(Signature::name).cloned().collect(),
        }
    }

    #[must_use]
    pub fn methods(&self, key: &TraitKey) -> BTreeSet<&MethodName> {
        self.0.get(key).into_iter().flatten().collect()
    }

    #[must_use]
    pub fn usable<'keys>(&self, keys: impl IntoIterator<Item = &'keys TraitKey>) -> Tally {
        keys.into_iter().map(|key| Tally::counting(self.methods(key))).sum()
    }
}
