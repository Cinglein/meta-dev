mod workflow_run;

use meta_gh::{Conclusion, Outcome};
use serde::Deserialize;

use crate::workflow_name::WorkflowName;
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
    pub fn run_of(&self, workflow: &WorkflowName) -> Option<&Self> {
        self.run.as_ref().filter(|run| *run == workflow).map(|_| self)
    }

    pub fn outcome(&self) -> Outcome {
        self.conclusion.map_or(Outcome::Undecided, Conclusion::outcome)
    }

    pub fn behind_a_newer_run(&self) -> Option<Conclusion> {
        self.conclusion.and_then(Conclusion::behind_a_newer_run)
    }
}
