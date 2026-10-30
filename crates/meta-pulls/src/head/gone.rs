use meta_schema::PullRequest;

use crate::departure::Departure;
use crate::live_pull::PullState;
use crate::transition::Transition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gone {
    pull_request: PullRequest,
    state: PullState,
}

impl Gone {
    pub(crate) fn new(pull_request: PullRequest, state: PullState) -> Self {
        Self { pull_request, state }
    }

    #[must_use]
    pub fn left(self) -> Transition {
        Transition::Left(Departure::new(self.pull_request, self.state))
    }
}
