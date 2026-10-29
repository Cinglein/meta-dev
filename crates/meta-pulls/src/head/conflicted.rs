use meta_schema::PullRequest;

use crate::transition::Transition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conflicted {
    pull_request: PullRequest,
}

impl Conflicted {
    pub(crate) fn new(pull_request: PullRequest) -> Self {
        Self { pull_request }
    }

    #[must_use]
    pub fn skipped(self) -> Transition {
        Transition::Conflict(self.pull_request)
    }
}
