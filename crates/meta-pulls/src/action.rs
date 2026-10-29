use std::ffi::OsString;
use std::fmt;

use meta_run::PullArgv;
use meta_schema::PullRequest;

use crate::live_pull::HeadCommit;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    CatchUp(PullRequest),
    Wait(PullRequest),
    Merge(PullRequest, HeadCommit),
}

impl Action {
    #[must_use]
    pub fn call(&self) -> Option<PullArgv> {
        match self {
            Self::CatchUp(_) => Some(PullArgv::UpdateBranch),
            Self::Merge(..) => Some(PullArgv::SquashMerge),
            Self::Wait(_) => None,
        }
    }

    #[must_use]
    pub fn operands(&self) -> Vec<OsString> {
        match self {
            Self::CatchUp(pull_request) | Self::Wait(pull_request) => vec![OsString::from(pull_request.to_string())],
            Self::Merge(pull_request, tested) => {
                vec![OsString::from(tested.as_ref()), OsString::from(pull_request.to_string())]
            }
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CatchUp(pull_request) => write!(formatter, "{pull_request} behind main: update branch"),
            Self::Wait(pull_request) => write!(formatter, "{pull_request} checks pending: wait"),
            Self::Merge(pull_request, tested) => {
                write!(formatter, "{pull_request} green and clean: squash-merge {tested}")
            }
        }
    }
}
