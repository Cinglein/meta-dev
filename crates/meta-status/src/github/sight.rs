use std::fmt;

use meta_pulls::LivePull;
use meta_schema::{Id, PullRequest, Task};

use crate::flag::Flag;

#[derive(Clone, Copy, Debug)]
pub enum Sight<'github> {
    Seen(&'github LivePull),
    Missing,
    Unreached,
}

impl Sight<'_> {
    #[must_use]
    pub fn flag(self, task: Id<Task>, pull_request: PullRequest, unheld: Option<()>) -> Option<Flag> {
        match self {
            Self::Seen(pull) => unheld.and(pull.red()).map(|_| Flag::RedUnheld(task, pull_request)),
            Self::Missing => Some(Flag::Missing(task, pull_request)),
            Self::Unreached => None,
        }
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
