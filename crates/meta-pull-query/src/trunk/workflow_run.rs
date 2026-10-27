use serde::Deserialize;

use super::workflow::Workflow;
use crate::workflow_name::WorkflowName;

#[derive(Clone, Debug, Deserialize)]
pub struct WorkflowRun {
    workflow: Workflow,
}

impl WorkflowRun {
    pub fn named(&self, name: &WorkflowName) -> bool {
        self.workflow.is(name)
    }
}
