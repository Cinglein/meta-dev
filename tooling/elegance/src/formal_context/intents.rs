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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::Intents;
    use crate::elegance_settings::EleganceSettings;
    use crate::signature::Signature;

    const ONE_INTENT: &str = "max_intents = 1";
    const INTENTS: [[&str; 2]; 2] = [["fn a()", "fn b()"], ["fn c()", "fn d()"]];

    #[test]
    fn the_cap_stops_the_enumeration_when_the_next_intent_would_exceed_it() {
        let settings: EleganceSettings = toml::from_str(ONE_INTENT).unwrap();
        let signature = |text: &&str| Signature::abstracted(&syn::parse_str::<syn::Signature>(text).unwrap(), None);
        let [first, second] = INTENTS.map(|texts| texts.iter().map(signature).collect::<BTreeSet<Signature>>());
        let cap = settings.max_intents();
        let after_first = Intents::default().met(&first, cap);
        let after_second = after_first.clone().met(&second, cap);
        assert_eq!((after_first.stopped(), after_second.stopped()), (None, Some(cap)));
        assert_eq!(after_second.into_found(), after_first.into_found());
    }
}
