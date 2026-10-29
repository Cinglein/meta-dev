use std::collections::BTreeSet;

use crate::signature::Signature;
use crate::trait_key::TraitKey;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TraitShape {
    methods: BTreeSet<Signature>,
    supertraits: BTreeSet<TraitKey>,
}

impl TraitShape {
    #[must_use]
    pub fn new(methods: BTreeSet<Signature>, supertraits: BTreeSet<TraitKey>) -> TraitShape {
        TraitShape { methods, supertraits }
    }

    #[must_use]
    pub fn methods(&self) -> &BTreeSet<Signature> {
        &self.methods
    }

    #[must_use]
    pub fn supertraits(&self) -> &BTreeSet<TraitKey> {
        &self.supertraits
    }
}
