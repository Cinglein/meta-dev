use meta_schema::PullRequest;

use crate::action::Action;
use crate::live_pull::HeadCommit;
use crate::transition::Transition;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Green {
    pull_request: PullRequest,
    tested: HeadCommit,
}

impl Green {
    pub(crate) fn new(pull_request: PullRequest, tested: HeadCommit) -> Self {
        Self { pull_request, tested }
    }

    #[must_use]
    pub fn merged(self) -> Transition {
        Transition::Act(Action::Merge(self.pull_request, self.tested))
    }
}
