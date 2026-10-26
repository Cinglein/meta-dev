use meta_pulls::RuledPull;
use meta_schema::PullRequest;

use crate::pulls::Pulls;
use crate::rulebook::Rulebook;

#[derive(Clone, Debug)]
pub struct LiveData {
    pulls: Pulls,
    rules: Rulebook,
}

impl LiveData {
    #[must_use]
    pub fn new(pulls: Pulls, rules: Rulebook) -> Self {
        Self { pulls, rules }
    }

    #[must_use]
    pub fn pull(&self, pull_request: PullRequest) -> Option<RuledPull<'_>> {
        let pull = self.pulls.pull(pull_request)?;
        self.rules.required(pull.base()).map(|required| RuledPull::new(pull, required))
    }

    #[must_use]
    pub fn orphans(&self, linked: &[PullRequest]) -> Vec<PullRequest> {
        self.pulls.open().orphans(linked)
    }

    #[must_use]
    pub fn truncated(&self) -> Option<()> {
        self.pulls.open().pull_requests().page_info().truncated()
    }
}
