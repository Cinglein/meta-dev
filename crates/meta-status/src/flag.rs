use std::fmt;

use meta_pull_query::WorkflowName;
use meta_schema::{Blocker, DueTime, Id, PullRequest, Task};
use serde::{Deserialize, Serialize};

use crate::github_failure::GithubFailure;
use crate::judgement::Judgement;
use crate::no_run::NoRun;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Flag {
    Unreached(GithubFailure),
    TruncatedOpen,
    Stalled(Id<Task>, DueTime),
    PastDue(Id<Task>, DueTime),
    Unanswered(Id<Task>, Blocker, DueTime),
    HeldDone(Id<Task>),
    TrunkRed(WorkflowName),
    RedUnheld(Id<Task>, PullRequest),
    Missing(Id<Task>, PullRequest),
    TruncatedChecks(Id<Task>, PullRequest),
    Unrun(Id<Task>, PullRequest, NoRun),
    Orphan(PullRequest),
}

impl Flag {
    #[must_use]
    pub fn stalled(&self) -> Option<DueTime> {
        match self {
            Self::Stalled(_, due) => Some(*due),
            Self::Unreached(_)
            | Self::TruncatedOpen
            | Self::PastDue(..)
            | Self::Unanswered(..)
            | Self::HeldDone(_)
            | Self::TrunkRed(_)
            | Self::RedUnheld(..)
            | Self::Missing(..)
            | Self::TruncatedChecks(..)
            | Self::Unrun(..)
            | Self::Orphan(_) => None,
        }
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreached(failure) => write!(formatter, "unknown github: {failure}"),
            Self::TruncatedOpen => formatter.write_str("truncated: more than 100 open pull requests"),
            Self::Stalled(task, due) => write!(formatter, "stalled {task}: past due {due}, no holder"),
            Self::PastDue(task, due) => write!(formatter, "past due {task}: due {due}"),
            Self::Unanswered(task, on, due) => write!(formatter, "unanswered {task} on {on}: reply due {due}"),
            Self::HeldDone(task) => write!(formatter, "done {task}: held, every linked pr merged or closed"),
            Self::TrunkRed(workflow) => write!(formatter, "trunk red {workflow}: latest completed run failed"),
            Self::RedUnheld(task, pull_request) => write!(formatter, "red {pull_request} {task}: no holder"),
            Self::Missing(task, pull_request) => write!(formatter, "unknown {pull_request} {task}: not on github"),
            Self::TruncatedChecks(task, pull_request) => {
                write!(formatter, "truncated {pull_request} {task}: more than 100 checks")
            }
            Self::Unrun(task, pull_request, reason) => write!(formatter, "waiting {pull_request} {task}: {reason}"),
            Self::Orphan(pull_request) => write!(formatter, "orphan {pull_request}: open, no task"),
        }
    }
}

impl Flag {
    pub fn judgement(&self) -> Judgement {
        match self {
            Self::Orphan(_) => Judgement::Noted,
            Self::Unreached(_)
            | Self::TruncatedOpen
            | Self::Stalled(..)
            | Self::PastDue(..)
            | Self::Unanswered(..)
            | Self::HeldDone(_)
            | Self::TrunkRed(_)
            | Self::RedUnheld(..)
            | Self::Missing(..)
            | Self::TruncatedChecks(..)
            | Self::Unrun(..) => Judgement::Needed,
        }
    }
}

impl Flag {
    pub fn repeats(&self, earlier: &Self) -> Option<()> {
        match (self, earlier) {
            (Self::Unreached(_), Self::Unreached(_)) => Some(()),
            _ => (self == earlier).then_some(()),
        }
    }
}
