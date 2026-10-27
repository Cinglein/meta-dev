mod conclusion;
mod workflow_run;

use serde::Deserialize;

use crate::trunk_verdict::TrunkVerdict;
use crate::workflow_name::WorkflowName;
use conclusion::Conclusion;
use workflow_run::WorkflowRun;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Suite {
    #[serde(default)]
    conclusion: Option<Conclusion>,
    #[serde(default, rename = "workflowRun")]
    run: Option<WorkflowRun>,
}

impl Suite {
    pub fn verdict(&self, workflow: &WorkflowName) -> Option<TrunkVerdict> {
        self.run.as_ref().filter(|run| *run == workflow)?;
        self.conclusion?.verdict()
    }
}
