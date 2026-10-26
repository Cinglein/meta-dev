use std::fmt;

use meta_pulls::{PullState, RuledPull};
use meta_schema::{Id, PullRequest, Task};

use crate::ending::Ending;
use crate::flag::Flag;
use crate::lifetime::Lifetime;

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
    pub fn lifetime(self, pull_request: PullRequest) -> Lifetime {
        match self {
            Self::Seen(pull) => match pull.state() {
                PullState::Open => Lifetime::Open(pull_request),
                PullState::Merged => Lifetime::Ended(Ending::Merged(pull_request)),
                PullState::Closed => Lifetime::Ended(Ending::Closed(pull_request)),
            },
            Self::Missing | Self::Unreached => Lifetime::Unknown(pull_request),
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
