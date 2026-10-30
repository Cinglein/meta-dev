use meta_schema::PullRequest;

use crate::failing_checks::FailingChecks;
use crate::stop::Stop;
use crate::transition::Transition;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Red {
    pull_request: PullRequest,
    checks: FailingChecks,
}

impl Red {
    pub(crate) fn new(pull_request: PullRequest, checks: FailingChecks) -> Self {
        Self { pull_request, checks }
    }

    #[must_use]
    pub fn stopped(self) -> Transition {
        Transition::Stop(Stop::Red { pull_request: self.pull_request, checks: self.checks })
    }
}
