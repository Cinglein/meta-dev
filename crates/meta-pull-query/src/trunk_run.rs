use std::fmt;

use crate::trunk_verdict::TrunkVerdict;
use crate::workflow_name::WorkflowName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrunkRun {
    workflow: WorkflowName,
    verdict: TrunkVerdict,
}

impl TrunkRun {
    #[must_use]
    pub fn new(workflow: WorkflowName, verdict: TrunkVerdict) -> Self {
        Self { workflow, verdict }
    }

    #[must_use]
    pub fn workflow(&self) -> &WorkflowName {
        &self.workflow
    }

    #[must_use]
    pub fn verdict(&self) -> TrunkVerdict {
        self.verdict
    }
}

impl fmt::Display for TrunkRun {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.workflow, self.verdict)
    }
}
