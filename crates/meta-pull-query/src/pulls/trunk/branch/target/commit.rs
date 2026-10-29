mod suite;

use meta_pulls::Page;
use serde::Deserialize;

use crate::workflow_name::WorkflowName;
pub use suite::Suite;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Commit {
    #[serde(rename = "checkSuites")]
    suites: Page<Suite>,
}

impl Commit {
    pub fn run_of(&self, workflow: &WorkflowName) -> Option<&Suite> {
        self.suites.nodes().find_map(|suite| suite.run_of(workflow))
    }
}
