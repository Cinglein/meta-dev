mod intents;

use std::collections::{BTreeMap, BTreeSet};

use crate::concept::Concept;
use crate::concept_report::ConceptReport;
use crate::facts::Facts;
use crate::impl_fact::ImplFact;
use crate::implementations::Implementations;
use crate::intent_cap::IntentCap;
use crate::signature::Signature;
use crate::tally::Tally;
use crate::trait_catalog::TraitCatalog;
use crate::type_key::TypeKey;
use intents::Intents;

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
    pub fn concepts(&self, catalog: &TraitCatalog, implementations: &Implementations, cap: IntentCap) -> ConceptReport {
        let intents = self
            .0
            .values()
            .filter(|intent| Tally::counting(*intent).reaches(Tally::PAIR))
            .fold(Intents::default(), |intents, intent| intents.met(intent, cap));
        let capped = intents.stopped();
        let mut concepts: Vec<Concept> = intents
            .into_found()
            .into_iter()
            .map(|intent| (self.extent(&intent), intent))
            .filter(|(extent, _)| Tally::counting(extent).reaches(Tally::PAIR))
            .map(|(extent, intent)| Concept::named(extent, intent, catalog, implementations))
            .collect();
        concepts.sort_by(Concept::larger_first);
        ConceptReport::new(concepts, capped)
    }

    #[must_use]
    pub fn extent(&self, intent: &BTreeSet<Signature>) -> BTreeSet<TypeKey> {
        self.0.iter().filter(|(_, signatures)| intent.is_subset(signatures)).map(|(owner, _)| owner.clone()).collect()
    }
}
