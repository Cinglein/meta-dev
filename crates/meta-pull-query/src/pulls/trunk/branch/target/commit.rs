mod suite;

use meta_pulls::Page;
use serde::Deserialize;

use crate::trunk_verdict::TrunkVerdict;
use crate::workflow_name::WorkflowName;
use suite::Suite;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Commit {
    #[serde(rename = "checkSuites")]
    suites: Page<Suite>,
}

impl Commit {
    pub fn verdict(&self, workflow: &WorkflowName) -> Option<TrunkVerdict> {
        self.suites.nodes().find_map(|suite| suite.verdict(workflow))
    }
}
