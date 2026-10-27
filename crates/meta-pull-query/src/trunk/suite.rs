use serde::Deserialize;

use super::conclusion::Conclusion;
use super::workflow_run::WorkflowRun;
use crate::trunk_verdict::TrunkVerdict;
use crate::workflow_name::WorkflowName;

#[derive(Clone, Debug, Deserialize)]
pub struct Suite {
    #[serde(default)]
    conclusion: Option<Conclusion>,
    #[serde(default, rename = "workflowRun")]
    run: Option<WorkflowRun>,
}

impl Suite {
    pub fn verdict(&self, workflow: &WorkflowName) -> Option<TrunkVerdict> {
        self.run.as_ref().filter(|run| run.named(workflow))?;
        self.conclusion?.verdict()
    }
}
