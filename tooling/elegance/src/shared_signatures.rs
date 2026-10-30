use crate::facts::Facts;
use crate::finding::Finding;
use crate::findings::Findings;
use crate::formal_context::FormalContext;
use crate::listed::Listed;
use crate::signature::Signature;
use crate::tally::Tally;
use crate::trait_key::TraitKey;

#[derive(Debug)]
pub struct SharedSignatures;

impl SharedSignatures {
    #[must_use]
    pub fn measured(facts: &Facts) -> Findings<Signature, Listed<TraitKey>> {
        let context = FormalContext::new(
            facts.traits().iter().map(|(key, shape)| (key.clone(), shape.methods().clone())).collect(),
        );
        Findings::new(
            context
                .extents()
                .into_iter()
                .filter(|(_, traits)| Tally::counting(traits).reaches(Tally::PAIR))
                .map(|(signature, traits)| Finding::new(signature, Listed::new(traits))),
        )
    }
}
