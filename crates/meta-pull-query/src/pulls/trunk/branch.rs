mod target;

use serde::Deserialize;

use crate::trunk_verdict::TrunkVerdict;
use crate::workflow_name::WorkflowName;
use target::Target;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Branch {
    target: Target,
}

impl Branch {
    pub fn verdict(&self, workflow: &WorkflowName) -> TrunkVerdict {
        self.target.verdict(workflow).unwrap_or(TrunkVerdict::Unknown)
    }
}
