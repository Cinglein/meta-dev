mod intents;

use std::collections::{BTreeMap, BTreeSet};

use crate::concept::Concept;
use crate::concept_report::ConceptReport;
use crate::facts::Facts;
use crate::field::Field;
use crate::impl_fact::ImplFact;
use crate::intent_cap::IntentCap;
use crate::reduction::Reduction;
use crate::signature::Signature;
use crate::tally::Tally;
use crate::type_key::TypeKey;
use intents::Intents;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormalContext<Object, Attribute>(BTreeMap<Object, BTreeSet<Attribute>>);

impl FormalContext<TypeKey, Signature> {
    #[must_use]
    pub fn incidence(facts: &Facts) -> FormalContext<TypeKey, Signature> {
        FormalContext(
            facts
                .types()
                .iter()
                .map(|owner| (owner.clone(), facts.impls_on(owner).flat_map(ImplFact::bodies).cloned().collect()))
                .collect(),
        )
    }
}

impl FormalContext<TypeKey, Field> {
    #[must_use]
    pub fn fields(facts: &Facts) -> FormalContext<TypeKey, Field> {
        FormalContext(
            facts.structure().layouts().iter().map(|(owner, layout)| (owner.clone(), layout.named().clone())).collect(),
        )
    }
}

impl<Object: Ord + Clone, Attribute: Ord + Clone> FormalContext<Object, Attribute> {
    #[must_use]
    pub fn new(incidence: BTreeMap<Object, BTreeSet<Attribute>>) -> FormalContext<Object, Attribute> {
        FormalContext(incidence)
    }

    #[must_use]
    pub fn concepts(&self, cap: IntentCap) -> ConceptReport<Object, Attribute> {
        let intents = self
            .0
            .values()
            .filter(|intent| Tally::counting(*intent).reaches(Tally::PAIR))
            .fold(Intents::default(), |intents, intent| intents.met(intent, cap));
        let capped = intents.stopped();
        let mut concepts: Vec<Concept<Object, Attribute>> = intents
            .into_found()
            .into_iter()
            .map(|intent| (self.extent(&intent), intent))
            .filter(|(extent, _)| Tally::counting(extent).reaches(Tally::PAIR))
            .map(|(extent, intent)| Concept::found(extent, intent))
            .collect();
        concepts.sort_by(Concept::larger_first);
        ConceptReport::new(concepts, capped)
    }

    #[must_use]
    pub fn extent(&self, intent: &BTreeSet<Attribute>) -> BTreeSet<Object> {
        self.0.iter().filter(|(_, attributes)| intent.is_subset(attributes)).map(|(owner, _)| owner.clone()).collect()
    }

    #[must_use]
    pub fn extents(&self) -> BTreeMap<Attribute, BTreeSet<Object>> {
        self.0.iter().flat_map(|(object, attributes)| attributes.iter().map(move |attribute| (attribute, object))).fold(
            BTreeMap::new(),
            |mut extents, (attribute, object)| {
                extents.entry(attribute.clone()).or_insert_with(BTreeSet::new).insert(object.clone());
                extents
            },
        )
    }

    #[must_use]
    pub fn reducible(&self) -> BTreeMap<Attribute, Reduction<Attribute>> {
        let extents = self.extents();
        extents
            .iter()
            .filter_map(|(attribute, extent)| {
                Reduction::found(attribute, extent, &extents).map(|reduction| (attribute.clone(), reduction))
            })
            .collect()
    }
}
