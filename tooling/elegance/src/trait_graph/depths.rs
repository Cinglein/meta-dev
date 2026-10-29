use std::collections::BTreeMap;

use crate::tally::Tally;
use crate::trait_key::TraitKey;

#[derive(Clone, Debug, Default)]
pub struct Depths<'graph>(BTreeMap<&'graph TraitKey, Tally>);

impl<'graph> Depths<'graph> {
    #[must_use]
    pub fn settled_depth(&self, key: &TraitKey) -> Option<Tally> {
        self.0.get(key).copied()
    }

    #[must_use]
    pub fn settle(self, key: &'graph TraitKey, depth: Tally) -> Depths<'graph> {
        Depths(self.0.into_iter().chain(std::iter::once((key, depth))).collect())
    }
}
