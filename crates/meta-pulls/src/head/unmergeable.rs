use meta_schema::PullRequest;

use crate::live_pull::MergeState;
use crate::stop::Stop;
use crate::transition::Transition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unmergeable {
    pull_request: PullRequest,
    state: MergeState,
}

impl Unmergeable {
    pub(crate) fn new(pull_request: PullRequest, state: MergeState) -> Self {
        Self { pull_request, state }
    }

    #[must_use]
    pub fn stopped(self) -> Transition {
        Transition::Stop(Stop::NotMergeable { pull_request: self.pull_request, state: self.state })
    }
}
