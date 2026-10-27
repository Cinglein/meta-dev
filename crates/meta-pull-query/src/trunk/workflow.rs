use serde::Deserialize;

use crate::workflow_name::WorkflowName;

#[derive(Clone, Debug, Deserialize)]
pub struct Workflow {
    name: WorkflowName,
}

impl Workflow {
    pub fn is(&self, name: &WorkflowName) -> bool {
        self.name == *name
    }
}
