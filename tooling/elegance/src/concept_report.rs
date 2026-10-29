use std::fmt;

use crate::concept::Concept;
use crate::naming::Naming;
use crate::tally::Tally;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConceptReport(Vec<Concept>);

impl ConceptReport {
    #[must_use]
    pub fn new(concepts: Vec<Concept>) -> ConceptReport {
        ConceptReport(concepts)
    }

    #[must_use]
    pub fn counted(&self, wanted: impl Fn(&Naming) -> bool) -> Tally {
        Tally::counting(self.0.iter().filter(|concept| wanted(concept.naming())))
    }

    pub fn ranked(&self, formatter: &mut fmt::Formatter<'_>, wanted: &Naming) -> fmt::Result {
        writeln!(formatter, "  largest, {wanted}:")?;
        self.0
            .iter()
            .filter(|concept| concept.naming() == wanted)
            .take(Tally::SHOWN)
            .enumerate()
            .try_for_each(|(rank, concept)| writeln!(formatter, "  {}. {concept}", rank + 1))
    }
}

impl fmt::Display for ConceptReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let named = self.counted(|naming| matches!(naming, Naming::Trait(_)));
        let composed = self.counted(|naming| *naming == Naming::Composed);
        let unnamed = self.counted(|naming| *naming == Naming::Missing);
        writeln!(formatter, "missing abstractions: maximal (types, signatures) concepts with at least 2 of each")?;
        writeln!(
            formatter,
            "  concepts {}: named by one trait {named}, by a composition only {composed}, unnamed {unnamed}",
            self.0.len()
        )?;
        writeln!(formatter, "  missing, one trait must name it: {}", composed + unnamed)?;
        writeln!(formatter, "  missing, a composition may name it: {unnamed}")?;
        self.ranked(formatter, &Naming::Missing)?;
        self.ranked(formatter, &Naming::Composed)
    }
}
