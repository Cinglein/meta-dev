mod sight;

use meta_pull_query::LiveData;
use meta_schema::PullRequest;

use crate::github_failure::GithubFailure;
use sight::Sight;

#[derive(Clone, Debug)]
pub enum Github {
    Reached(LiveData),
    Unreached(GithubFailure),
}

impl Github {
    #[must_use]
    pub fn sight(&self, pull_request: PullRequest) -> Sight<'_> {
        match self {
            Self::Reached(data) => data.pull(pull_request).map_or(Sight::Missing, Sight::Seen),
            Self::Unreached(_) => Sight::Unreached,
        }
    }

    #[must_use]
    pub fn orphans(&self, linked: &[PullRequest]) -> Vec<PullRequest> {
        match self {
            Self::Reached(data) => data.open().orphans(linked),
            Self::Unreached(_) => Vec::new(),
        }
    }

    #[must_use]
    pub fn failure(&self) -> Option<&GithubFailure> {
        match self {
            Self::Reached(_) => None,
            Self::Unreached(failure) => Some(failure),
        }
    }
}
