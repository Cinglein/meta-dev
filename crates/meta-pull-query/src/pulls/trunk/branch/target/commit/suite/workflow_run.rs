mod workflow;

use serde::Deserialize;

use crate::workflow_name::WorkflowName;
use workflow::Workflow;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRun {
    workflow: Workflow,
}

impl PartialEq<WorkflowName> for WorkflowRun {
    fn eq(&self, other: &WorkflowName) -> bool {
        self.workflow == *other
    }
}
