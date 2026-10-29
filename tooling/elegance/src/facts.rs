use std::collections::{BTreeMap, BTreeSet};

use crate::impl_fact::ImplFact;
use crate::trait_key::TraitKey;
use crate::trait_shape::TraitShape;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Facts {
    types: BTreeSet<TypeKey>,
    traits: BTreeMap<TraitKey, TraitShape>,
    impls: Vec<ImplFact>,
}

impl Facts {
    #[must_use]
    pub fn new(types: BTreeSet<TypeKey>, traits: BTreeMap<TraitKey, TraitShape>, impls: Vec<ImplFact>) -> Facts {
        Facts { types, traits, impls }
    }

    #[must_use]
    pub fn types(&self) -> &BTreeSet<TypeKey> {
        &self.types
    }

    #[must_use]
    pub fn traits(&self) -> &BTreeMap<TraitKey, TraitShape> {
        &self.traits
    }

    #[must_use]
    pub fn impls(&self) -> &[ImplFact] {
        &self.impls
    }

    pub fn impls_of<'facts>(
        &'facts self,
        implemented: &TraitKey,
    ) -> impl Iterator<Item = &'facts ImplFact> + use<'facts> {
        let wanted = implemented.clone();
        self.impls.iter().filter(move |fact| fact.kind().implemented() == Some(&wanted))
    }

    #[must_use]
    pub fn concrete_targets(&self, implemented: &TraitKey) -> BTreeSet<&TypeKey> {
        self.impls_of(implemented).filter_map(|fact| fact.kind().target()).collect()
    }

    pub fn impls_on<'facts>(&'facts self, target: &TypeKey) -> impl Iterator<Item = &'facts ImplFact> + use<'facts> {
        let wanted = target.clone();
        self.impls.iter().filter(move |fact| fact.kind().target() == Some(&wanted))
    }
}
