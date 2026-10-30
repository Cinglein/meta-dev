use std::fmt;

use meta_schema::PullRequest;

use crate::action::Action;
use crate::failing_checks::FailingChecks;
use crate::gh_refusal::GhRefusal;
use crate::live_pull::{BaseBranch, MergeState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stop {
    Red { pull_request: PullRequest, checks: FailingChecks },
    ChecksTimeout(PullRequest),
    NotMergeable { pull_request: PullRequest, state: MergeState },
    OffTrunk { pull_request: PullRequest, base: BaseBranch },
    Unguarded(BaseBranch),
    Incomplete(PullRequest),
    Missing(PullRequest),
    Unread { pull_request: PullRequest, refusal: GhRefusal },
    Refused { action: Action, refusal: GhRefusal },
    TrunkUnread(GhRefusal),
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
            Self::OffTrunk { pull_request, base } => {
                write!(formatter, "stop: {pull_request} targets {base}, not the default branch")
            }
            Self::Unguarded(base) => {
                write!(formatter, "stop: {base} does not require branches to be up to date; nothing lands untested")
            }
            Self::Incomplete(pull_request) => {
                write!(formatter, "stop: {pull_request} reply has no merge state or head commit")
            }
            Self::Missing(pull_request) => {
                write!(formatter, "stop: {pull_request} not in the reply, or its base branch rules were not read")
            }
            Self::Unread { pull_request, refusal } => write!(formatter, "stop: {pull_request} poll failed: {refusal}"),
            Self::Refused { action, refusal } => write!(formatter, "stop: {action} failed: {refusal}"),
            Self::TrunkUnread(refusal) => write!(formatter, "stop: default branch rules unread: {refusal}"),
        }
    }
}
