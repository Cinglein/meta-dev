mod prior_allowance;

use serde::Deserialize;
use task::BaselineFile;

use prior_allowance::PriorAllowance;

#[derive(Debug, Default, Deserialize)]
pub struct PriorBudget {
    #[serde(default)]
    allowance: Option<PriorAllowance>,
}

impl PriorBudget {
    pub fn baseline(&self) -> Option<&BaselineFile> {
        self.allowance.as_ref().and_then(PriorAllowance::baseline)
    }
}
