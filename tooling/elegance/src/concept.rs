use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

use crate::implementations::Implementations;
use crate::naming::Naming;
use crate::signature::Signature;
use crate::tally::Tally;
use crate::trait_catalog::TraitCatalog;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Concept {
    extent: BTreeSet<TypeKey>,
    intent: BTreeSet<Signature>,
    naming: Naming,
}

impl Concept {
    #[must_use]
    pub fn named(
        extent: BTreeSet<TypeKey>,
        intent: BTreeSet<Signature>,
        catalog: &TraitCatalog,
        implementations: &Implementations,
    ) -> Concept {
        let covered = |methods: BTreeSet<&Signature>| intent.iter().all(|signature| methods.contains(signature));
        let shared = implementations.shared_by(&extent);
        let single = shared.iter().find(|key| covered(catalog.methods(key))).map(|key| Naming::Trait((*key).clone()));
        let composed: BTreeSet<&Signature> = shared.iter().flat_map(|key| catalog.methods(key)).collect();
        let naming = single.unwrap_or(if covered(composed) { Naming::Composed } else { Naming::Missing });
        Concept { extent, intent, naming }
    }

    #[must_use]
    pub fn naming(&self) -> &Naming {
        &self.naming
    }

    #[must_use]
    pub fn area(&self) -> Tally {
        Tally::counting(&self.extent) * Tally::counting(&self.intent)
    }

    #[must_use]
    pub fn larger_first(&self, other: &Concept) -> Ordering {
        other
            .area()
            .cmp(&self.area())
            .then_with(|| other.extent.len().cmp(&self.extent.len()))
            .then_with(|| self.extent.cmp(&other.extent))
            .then_with(|| self.intent.cmp(&other.intent))
    }
}

impl fmt::Display for Concept {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let types: Vec<String> = self.extent.iter().map(ToString::to_string).collect();
        let signatures: Vec<String> = self.intent.iter().map(ToString::to_string).collect();
        write!(
            formatter,
            "{} = {} types x {} signatures\n    types: {}\n    share: {}",
            self.area(),
            self.extent.len(),
            self.intent.len(),
            types.join(", "),
            signatures.join("; ")
        )
    }
}
