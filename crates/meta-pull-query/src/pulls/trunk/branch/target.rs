mod commit;

use meta_gh::Conclusion;
use meta_pulls::Page;
use serde::Deserialize;

use crate::trunk_verdict::TrunkVerdict;
use crate::workflow_name::WorkflowName;
use commit::{Commit, Suite};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    history: Page<Commit>,
}

impl Target {
    pub fn verdict(&self, workflow: &WorkflowName) -> Option<TrunkVerdict> {
        let mut runs = self.history.nodes().filter_map(|commit| commit.run_of(workflow));
        let newest = TrunkVerdict::decided(runs.next()?.outcome());
        let older = |run: &Suite| run.behind_a_newer_run().map(Conclusion::outcome).and_then(TrunkVerdict::decided);
        newest.or_else(|| runs.find_map(older))
    }
}
