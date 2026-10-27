mod branch;
mod commit;
mod conclusion;
mod suite;
mod target;
mod workflow;
mod workflow_run;

use serde::Deserialize;

use crate::trunk_verdict::TrunkVerdict;
use crate::workflow_name::WorkflowName;
use branch::Branch;

#[derive(Clone, Debug, Deserialize)]
pub struct Trunk {
    #[serde(rename = "defaultBranchRef")]
    branch: Option<Branch>,
}

impl Trunk {
    #[must_use]
    pub fn verdict(&self, workflow: &WorkflowName) -> TrunkVerdict {
        let mut suites = self.branch.iter().flat_map(Branch::suites);
        suites.find_map(|suite| suite.verdict(workflow)).unwrap_or(TrunkVerdict::Unknown)
    }
}
