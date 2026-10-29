use std::collections::BTreeSet;

use crate::intent_cap::IntentCap;
use crate::signature::Signature;
use crate::tally::Tally;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Intents {
    found: BTreeSet<BTreeSet<Signature>>,
    stopped: Option<IntentCap>,
}

impl Intents {
    #[must_use]
    pub fn met(self, intent: &BTreeSet<Signature>, cap: IntentCap) -> Intents {
        let fresh: BTreeSet<BTreeSet<Signature>> = self
            .found
            .iter()
            .map(|other| other.intersection(intent).cloned().collect::<BTreeSet<Signature>>())
            .filter(|meet| Tally::counting(meet).reaches(Tally::PAIR))
            .chain(std::iter::once(intent.clone()))
            .filter(|candidate| !self.found.contains(candidate))
            .collect();
        if self.stopped.is_none() && cap.allows(self.found.len() + fresh.len()) {
            Intents { found: self.found.into_iter().chain(fresh).collect(), stopped: None }
        } else {
            Intents { found: self.found, stopped: self.stopped.or(Some(cap)) }
        }
    }

    #[must_use]
    pub fn stopped(&self) -> Option<IntentCap> {
        self.stopped
    }

    #[must_use]
    pub fn into_found(self) -> BTreeSet<BTreeSet<Signature>> {
        self.found
    }
}
