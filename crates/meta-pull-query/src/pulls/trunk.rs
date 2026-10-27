mod branch;

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
        self.branch.as_ref().map_or(TrunkVerdict::Unknown, |branch| branch.verdict(workflow))
    }
}
