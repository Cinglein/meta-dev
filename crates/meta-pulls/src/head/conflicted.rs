use meta_schema::PullRequest;

use crate::detour::Detour;
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
    pub fn detoured(self) -> Transition {
        Transition::Detour(Detour::Conflict(self.pull_request))
    }
}
