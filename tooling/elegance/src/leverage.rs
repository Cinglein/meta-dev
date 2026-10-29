use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

use crate::facts::Facts;
use crate::hundredths::Hundredths;
use crate::impl_fact::ImplFact;
use crate::implementations::Implementations;
use crate::method_name::MethodName;
use crate::signature::Signature;
use crate::tally::Tally;
use crate::trait_catalog::TraitCatalog;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leverage {
    owner: TypeKey,
    usable: Tally,
    written: Tally,
    ratio: Hundredths,
}

impl Leverage {
    #[must_use]
    pub fn measured(
        owner: &TypeKey,
        facts: &Facts,
        catalog: &TraitCatalog,
        implementations: &Implementations,
    ) -> Leverage {
        let inherent: BTreeSet<&MethodName> = facts
            .impls_on(owner)
            .filter(|fact| fact.kind().implemented().is_none())
            .flat_map(ImplFact::bodies)
            .map(Signature::name)
            .collect();
        let usable = Tally::counting(inherent) + catalog.usable(implementations.traits_of(owner));
        let written = Tally::counting(facts.impls_on(owner).flat_map(ImplFact::bodies));
        Leverage { owner: owner.clone(), usable, written, ratio: usable.per(written) }
    }

    #[must_use]
    pub fn written(&self) -> Tally {
        self.written
    }

    #[must_use]
    pub fn ratio(&self) -> Hundredths {
        self.ratio
    }

    #[must_use]
    pub fn lowest_first(&self, other: &Leverage) -> Ordering {
        (self.usable * other.written)
            .cmp(&(other.usable * self.written))
            .then_with(|| other.written.cmp(&self.written))
            .then_with(|| self.owner.cmp(&other.owner))
    }
}

impl fmt::Display for Leverage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} ({} usable / {} written)", self.owner, self.ratio, self.usable, self.written)
    }
}
