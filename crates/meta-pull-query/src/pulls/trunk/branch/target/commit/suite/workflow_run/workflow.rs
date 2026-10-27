use serde::Deserialize;

use crate::workflow_name::WorkflowName;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    name: WorkflowName,
}

impl PartialEq<WorkflowName> for Workflow {
    fn eq(&self, other: &WorkflowName) -> bool {
        self.name == *other
    }
}
