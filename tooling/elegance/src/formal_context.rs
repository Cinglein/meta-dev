use std::collections::{BTreeMap, BTreeSet};

use crate::concept::Concept;
use crate::facts::Facts;
use crate::impl_fact::ImplFact;
use crate::implementations::Implementations;
use crate::signature::Signature;
use crate::tally::Tally;
use crate::trait_catalog::TraitCatalog;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormalContext(BTreeMap<TypeKey, BTreeSet<Signature>>);

impl FormalContext {
    #[must_use]
    pub fn incidence(facts: &Facts) -> FormalContext {
        FormalContext(
            facts
                .types()
                .iter()
                .map(|owner| (owner.clone(), facts.impls_on(owner).flat_map(ImplFact::bodies).cloned().collect()))
                .collect(),
        )
    }

    #[must_use]
    pub fn concepts(&self, catalog: &TraitCatalog, implementations: &Implementations) -> Vec<Concept> {
        let intents = self
            .0
            .values()
            .filter(|intent| Tally::counting(*intent).reaches(Tally::PAIR))
            .fold(BTreeSet::new(), Self::meets);
        let mut concepts: Vec<Concept> = intents
            .into_iter()
            .map(|intent| (self.extent(&intent), intent))
            .filter(|(extent, _)| Tally::counting(extent).reaches(Tally::PAIR))
            .map(|(extent, intent)| Concept::named(extent, intent, catalog, implementations))
            .collect();
        concepts.sort_by(Concept::larger_first);
        concepts
    }

    #[must_use]
    pub fn meets(found: BTreeSet<BTreeSet<Signature>>, intent: &BTreeSet<Signature>) -> BTreeSet<BTreeSet<Signature>> {
        let meets: Vec<BTreeSet<Signature>> = found
            .iter()
            .map(|other| other.intersection(intent).cloned().collect::<BTreeSet<Signature>>())
            .filter(|meet| Tally::counting(meet).reaches(Tally::PAIR))
            .collect();
        found.into_iter().chain(meets).chain(std::iter::once(intent.clone())).collect()
    }

    #[must_use]
    pub fn extent(&self, intent: &BTreeSet<Signature>) -> BTreeSet<TypeKey> {
        self.0.iter().filter(|(_, signatures)| intent.is_subset(signatures)).map(|(owner, _)| owner.clone()).collect()
    }
}
