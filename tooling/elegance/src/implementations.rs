use std::collections::{BTreeMap, BTreeSet};

use crate::facts::Facts;
use crate::trait_key::TraitKey;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Implementations(BTreeMap<TypeKey, BTreeSet<TraitKey>>);

impl Implementations {
    #[must_use]
    pub fn closed(facts: &Facts) -> Implementations {
        Implementations(facts.types().iter().map(|owner| (owner.clone(), Self::direct(facts, owner))).collect())
            .saturated(facts)
    }

    #[must_use]
    pub fn direct(facts: &Facts, owner: &TypeKey) -> BTreeSet<TraitKey> {
        facts.impls_on(owner).filter_map(|fact| fact.kind().implemented()).cloned().collect()
    }

    fn saturated(self, facts: &Facts) -> Implementations {
        let next = Implementations(
            self.0.iter().map(|(owner, traits)| (owner.clone(), Self::derived(facts, traits))).collect(),
        );
        if next == self { self } else { next.saturated(facts) }
    }

    #[must_use]
    pub fn derived(facts: &Facts, traits: &BTreeSet<TraitKey>) -> BTreeSet<TraitKey> {
        facts
            .impls()
            .iter()
            .filter(|fact| fact.kind().blanket_bounds().is_some_and(|bounds| bounds.is_subset(traits)))
            .filter_map(|fact| fact.kind().implemented())
            .chain(traits)
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn traits_of(&self, owner: &TypeKey) -> BTreeSet<&TraitKey> {
        self.0.get(owner).into_iter().flatten().collect()
    }

    #[must_use]
    pub fn shared_by<'types>(&self, owners: impl IntoIterator<Item = &'types TypeKey>) -> BTreeSet<&TraitKey> {
        owners
            .into_iter()
            .map(|owner| self.traits_of(owner))
            .reduce(|shared, traits| shared.intersection(&traits).copied().collect())
            .unwrap_or_default()
    }
}
