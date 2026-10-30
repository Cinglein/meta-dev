use meta_schema::PullRequest;

use crate::action::Action;
use crate::patience::Patience;
use crate::stop::Stop;
use crate::transition::Transition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Checking {
    pull_request: PullRequest,
}

impl Checking {
    pub(crate) fn new(pull_request: PullRequest) -> Self {
        Self { pull_request }
    }

    #[must_use]
    pub fn waited(self, patience: Patience) -> Transition {
        let waiting = Transition::Act(Action::Wait(self.pull_request));
        patience.spent().map_or(waiting, |()| Transition::Stop(Stop::ChecksTimeout(self.pull_request)))
    }
}
