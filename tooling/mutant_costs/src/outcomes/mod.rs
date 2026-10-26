mod outcome;

use std::collections::BTreeMap;

use outcome::Outcome;
use serde::Deserialize;

use crate::tally::Tally;
use task::CrateName;
use task::Failure;
use task::Millis;

#[derive(Debug, Deserialize)]
pub struct Outcomes {
    outcomes: Vec<Outcome>,
}

impl Outcomes {
    pub fn parse(json: &str) -> Result<Outcomes, Failure> {
        serde_json::from_str(json).map_err(Failure::MutantList)
    }

    #[must_use]
    pub fn tallied_into(&self, tallies: BTreeMap<CrateName, Tally>) -> BTreeMap<CrateName, Tally> {
        self.outcomes.iter().fold(tallies, |tallies, outcome| outcome.tallied_into(tallies))
    }

    #[must_use]
    pub fn baseline(&self) -> Millis {
        self.outcomes.iter().filter_map(Outcome::baseline).max().unwrap_or_default()
    }
}
