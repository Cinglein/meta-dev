use std::fmt;

use meta_schema::PullRequest;

use crate::live_pull::PullState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Departure {
    pull_request: PullRequest,
    state: PullState,
}

impl Departure {
    pub(crate) fn new(pull_request: PullRequest, state: PullState) -> Self {
        Self { pull_request, state }
    }

    #[must_use]
    pub fn landed(self) -> Option<PullRequest> {
        (self.state == PullState::Merged).then_some(self.pull_request)
    }
}

impl fmt::Display for Departure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}: next", self.pull_request, self.state)
    }
}
