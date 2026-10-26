use std::fmt;

use meta_pulls::RuledPull;
use meta_schema::{Id, PullRequest, Task};

use crate::flag::Flag;

#[derive(Clone, Copy, Debug)]
pub enum Sight<'github> {
    Seen(RuledPull<'github>),
    Missing,
    Unreached,
}

impl Sight<'_> {
    #[must_use]
    pub fn open(self) -> Option<()> {
        match self {
            Self::Seen(pull) => pull.open(),
            Self::Missing | Self::Unreached => None,
        }
    }

    #[must_use]
    pub fn flag(self, task: Id<Task>, pull_request: PullRequest, unheld: Option<()>) -> Option<Flag> {
        match self {
            Self::Seen(pull) => Self::seen_flag(pull, task, pull_request, unheld),
            Self::Missing => Some(Flag::Missing(task, pull_request)),
            Self::Unreached => None,
        }
    }

    fn seen_flag(pull: RuledPull<'_>, task: Id<Task>, pull_request: PullRequest, unheld: Option<()>) -> Option<Flag> {
        let truncated = pull.truncated().map(|_| Flag::TruncatedChecks(task, pull_request));
        truncated.or_else(|| unheld.and(pull.red()).map(|_| Flag::RedUnheld(task, pull_request)))
    }
}

impl fmt::Display for Sight<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Seen(pull) => write!(formatter, "{pull}"),
            Self::Missing | Self::Unreached => formatter.write_str("unknown"),
        }
    }
}
