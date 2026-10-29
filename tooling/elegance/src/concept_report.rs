use std::fmt;

use crate::concept::Concept;
use crate::field::Field;
use crate::implementations::Implementations;
use crate::intent_cap::IntentCap;
use crate::naming::Naming;
use crate::signature::Signature;
use crate::tally::Tally;
use crate::trait_catalog::TraitCatalog;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConceptReport<Object, Attribute> {
    concepts: Vec<Concept<Object, Attribute>>,
    capped: Option<IntentCap>,
}

impl ConceptReport<TypeKey, Signature> {
    #[must_use]
    pub fn named(self, catalog: &TraitCatalog, implementations: &Implementations) -> ConceptReport<TypeKey, Signature> {
        ConceptReport {
            concepts: self.concepts.into_iter().map(|concept| concept.named(catalog, implementations)).collect(),
            capped: self.capped,
        }
    }
}

impl<Object: Ord, Attribute: Ord> ConceptReport<Object, Attribute> {
    #[must_use]
    pub fn new(
        concepts: Vec<Concept<Object, Attribute>>,
        capped: Option<IntentCap>,
    ) -> ConceptReport<Object, Attribute> {
        ConceptReport { concepts, capped }
    }

    #[must_use]
    pub fn counted(&self, wanted: impl Fn(&Naming) -> bool) -> Tally {
        Tally::counting(self.concepts.iter().filter(|concept| wanted(concept.naming())))
    }
}

impl<Object: Ord + fmt::Display, Attribute: Ord + fmt::Display> ConceptReport<Object, Attribute> {
    pub fn ranked(&self, formatter: &mut fmt::Formatter<'_>, wanted: &Naming) -> fmt::Result {
        writeln!(formatter, "  largest, {wanted}:")?;
        self.concepts
            .iter()
            .filter(|concept| concept.naming() == wanted)
            .take(Tally::SHOWN)
            .enumerate()
            .try_for_each(|(rank, concept)| writeln!(formatter, "  {}. {concept}", rank + 1))
    }

    pub fn capped(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.capped.iter().try_for_each(|cap| {
            writeln!(
                formatter,
                "  INCOMPLETE: more than {cap} distinct intents; enumeration stopped there, so every count below is a lower bound"
            )
        })
    }
}

impl fmt::Display for ConceptReport<TypeKey, Signature> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let named = self.counted(|naming| matches!(naming, Naming::Trait(_)));
        let composed = self.counted(|naming| *naming == Naming::Composed);
        let unnamed = self.counted(|naming| *naming == Naming::Missing);
        writeln!(formatter, "B1 missing abstractions: maximal (types, signatures) concepts with at least 2 of each")?;
        self.capped(formatter)?;
        writeln!(
            formatter,
            "  concepts {}: named by one trait {named}, by a composition only {composed}, unnamed {unnamed}",
            self.concepts.len()
        )?;
        writeln!(formatter, "  missing, one trait must name it: {}", composed + unnamed)?;
        writeln!(formatter, "  missing, a composition may name it: {unnamed}")?;
        self.ranked(formatter, &Naming::Missing)?;
        self.ranked(formatter, &Naming::Composed)
    }
}

impl fmt::Display for ConceptReport<TypeKey, Field> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "B12 field concepts: maximal (types, named fields) concepts with at least 2 of each")?;
        writeln!(
            formatter,
            "  a field is its name and its written type; a type that nests the set names it, so its fields are not shared"
        )?;
        self.capped(formatter)?;
        writeln!(formatter, "  concepts {}", self.concepts.len())?;
        self.ranked(formatter, &Naming::Missing)
    }
}
