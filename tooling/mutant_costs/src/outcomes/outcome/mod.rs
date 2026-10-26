mod phase_result;
mod scenario;

use std::collections::BTreeMap;

use phase_result::PhaseResult;
use scenario::Scenario;
use serde::Deserialize;

use crate::tally::Tally;
use task::CrateName;
use task::Millis;

#[derive(Debug, Deserialize)]
pub struct Outcome {
    scenario: Scenario,
    phase_results: Vec<PhaseResult>,
}

impl Outcome {
    #[must_use]
    pub fn tallied_into(&self, mut tallies: BTreeMap<CrateName, Tally>) -> BTreeMap<CrateName, Tally> {
        match self.scenario.package() {
            Some(package) => {
                let tally = tallies.entry(package.clone()).or_default();
                *tally = tally.plus(Tally::one(PhaseResult::total(&self.phase_results)));
                tallies
            }
            None => tallies,
        }
    }

    #[must_use]
    pub fn baseline(&self) -> Option<Millis> {
        self.scenario.package().is_none().then(|| PhaseResult::total(&self.phase_results))
    }
}
