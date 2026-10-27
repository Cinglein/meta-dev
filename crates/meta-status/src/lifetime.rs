use std::fmt;

use meta_schema::PullRequest;

use crate::ending::Ending;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifetime {
    Ended(Ending),
    Open(PullRequest),
    Unlinked,
    Unknown(PullRequest),
    Unlogged,
}

impl Lifetime {
    #[must_use]
    pub fn of_pulls(pulls: &[Self]) -> Self {
        let unknown = pulls.iter().find(|lifetime| matches!(lifetime, Self::Unknown(_)));
        let open = || pulls.iter().find(|lifetime| matches!(lifetime, Self::Open(_)));
        let merged = || pulls.iter().find(|lifetime| matches!(lifetime, Self::Ended(Ending::Merged(_))));
        unknown.or_else(open).or_else(merged).or_else(|| pulls.first()).copied().unwrap_or(Self::Unlinked)
    }

    #[must_use]
    pub fn ending(self) -> Option<Ending> {
        match self {
            Self::Ended(ending) => Some(ending),
            Self::Open(_) | Self::Unlinked | Self::Unknown(_) | Self::Unlogged => None,
        }
    }
}

impl fmt::Display for Lifetime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ended(ending) => write!(formatter, "{ending}"),
            Self::Open(pull_request) => write!(formatter, "pr {pull_request} open"),
            Self::Unlinked => formatter.write_str("no pr linked"),
            Self::Unknown(pull_request) => write!(formatter, "pr {pull_request} unknown"),
            Self::Unlogged => formatter.write_str("task not in the log"),
        }
    }
}
