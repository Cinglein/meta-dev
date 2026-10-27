mod commit;

use meta_pulls::Page;
use serde::Deserialize;

use crate::trunk_verdict::TrunkVerdict;
use crate::workflow_name::WorkflowName;
use commit::Commit;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    history: Page<Commit>,
}

impl Target {
    pub fn verdict(&self, workflow: &WorkflowName) -> Option<TrunkVerdict> {
        self.history.nodes().find_map(|commit| commit.verdict(workflow))
    }
}
