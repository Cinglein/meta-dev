use std::fmt;

use meta_schema::PullRequest;

use crate::failing_checks::FailingChecks;
use crate::live_pull::MergeState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stop {
    Red { pull_request: PullRequest, checks: FailingChecks },
    ChecksTimeout(PullRequest),
    NotMergeable { pull_request: PullRequest, state: MergeState },
}

impl fmt::Display for Stop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Red { pull_request, checks } => write!(formatter, "stop: {pull_request} red: {checks}"),
            Self::ChecksTimeout(pull_request) => {
                write!(formatter, "stop: {pull_request} checks still pending at the deadline")
            }
            Self::NotMergeable { pull_request, state } => {
                write!(formatter, "stop: {pull_request} green but not mergeable: {state}")
            }
        }
    }
}
