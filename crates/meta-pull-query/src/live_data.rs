use meta_pulls::RuledPull;
use meta_schema::{Id, PullRequest, Task};

use crate::pulls::Pulls;
use crate::rulebook::Rulebook;
use crate::trunk_run::TrunkRun;
use crate::workflow_name::WorkflowName;

#[derive(Clone, Debug)]
pub struct LiveData {
    pulls: Box<Pulls>,
    rules: Rulebook,
    workflows: Vec<WorkflowName>,
}

impl LiveData {
    #[must_use]
    pub fn new(pulls: Pulls, rules: Rulebook) -> Self {
        Self { pulls: Box::new(pulls), rules, workflows: Vec::new() }
    }

    #[must_use]
    pub fn requiring(self, workflows: Vec<WorkflowName>) -> Self {
        Self { workflows, ..self }
    }

    #[must_use]
    pub fn pull(&self, pull_request: PullRequest) -> Option<RuledPull<'_>> {
        let pull = self.pulls.pull(pull_request)?;
        RuledPull::new(pull, self.rules.required(pull.base()))
    }

    #[must_use]
    pub fn headed(&self, task: Id<Task>) -> Vec<PullRequest> {
        self.pulls.open().headed(task)
    }

    #[must_use]
    pub fn orphans(&self, linked: &[PullRequest]) -> Vec<PullRequest> {
        self.pulls.open().orphans(linked)
    }

    #[must_use]
    pub fn trunk(&self) -> Vec<TrunkRun> {
        self.workflows.iter().cloned().map(|workflow| self.pulls.trunk_run(workflow)).collect()
    }

    #[must_use]
    pub fn truncated(&self) -> Option<()> {
        self.pulls.open().pull_requests().page_info().truncated()
    }
}
