use std::fmt;

use meta_schema::{DueTime, Id, PullRequest, Task};

use crate::github_failure::GithubFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Flag {
    Unreached(GithubFailure),
    PastDue(Id<Task>, DueTime),
    RedUnheld(Id<Task>, PullRequest),
    Missing(Id<Task>, PullRequest),
    Orphan(PullRequest),
}

impl fmt::Display for Flag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreached(failure) => write!(formatter, "unknown github: {failure}"),
            Self::PastDue(task, due) => write!(formatter, "stalled {task}: past due {due}, no holder"),
            Self::RedUnheld(task, pull_request) => write!(formatter, "red {pull_request} {task}: no holder"),
            Self::Missing(task, pull_request) => write!(formatter, "unknown {pull_request} {task}: not on github"),
            Self::Orphan(pull_request) => write!(formatter, "orphan {pull_request}: open, no task"),
        }
    }
}
