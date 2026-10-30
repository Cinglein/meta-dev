use std::collections::BTreeSet;

use crate::facts::Facts;
use crate::finding::Finding;
use crate::findings::Findings;
use crate::formal_context::FormalContext;
use crate::implementations::Implementations;
use crate::reduction::Reduction;
use crate::trait_key::TraitKey;

#[derive(Debug)]
pub struct ReducibleTraits;

impl ReducibleTraits {
    #[must_use]
    pub fn measured(facts: &Facts, implementations: &Implementations) -> Findings<TraitKey, Reduction<TraitKey>> {
        let blanketed = facts.blanketed();
        let eligible: BTreeSet<&TraitKey> = facts.traits().keys().filter(|key| !blanketed.contains(key)).collect();
        let context = FormalContext::new(
            facts
                .types()
                .iter()
                .map(|owner| {
                    let traits = implementations.traits_of(owner).into_iter().filter(|key| eligible.contains(key));
                    (owner.clone(), traits.cloned().collect())
                })
                .collect(),
        );
        Findings::new(context.reducible().into_iter().map(|(key, reduction)| Finding::new(key, reduction)))
    }
}
