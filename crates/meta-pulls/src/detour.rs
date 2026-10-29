use std::fmt;

use meta_schema::PullRequest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detour {
    Conflict(PullRequest),
}

impl Detour {
    #[must_use]
    pub fn pull_request(self) -> PullRequest {
        match self {
            Self::Conflict(pull_request) => pull_request,
        }
    }
}

impl fmt::Display for Detour {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict(pull_request) => write!(formatter, "{pull_request} conflicts with main: needs conflict fix"),
        }
    }
}
