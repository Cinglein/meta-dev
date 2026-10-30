use meta_schema::PullRequest;

use crate::action::Action;
use crate::transition::Transition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behind {
    pull_request: PullRequest,
}

impl Behind {
    pub(crate) fn new(pull_request: PullRequest) -> Self {
        Self { pull_request }
    }

    #[must_use]
    pub fn caught_up(self) -> Transition {
        Transition::Act(Action::CatchUp(self.pull_request))
    }
}
