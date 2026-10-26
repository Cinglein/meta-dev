use std::fmt;

use meta_schema::{DueTime, PullRequest};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    Merged(PullRequest),
    Closed(PullRequest),
    Stalled(DueTime),
}

impl fmt::Display for Ending {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Merged(pull_request) => write!(formatter, "pr {pull_request} merged"),
            Self::Closed(pull_request) => write!(formatter, "pr {pull_request} closed"),
            Self::Stalled(due) => write!(formatter, "stalled: past due {due}, no holder"),
        }
    }
}
