mod sight;

use meta_pull_query::{LiveData, TrunkRun, TrunkVerdict};
use meta_schema::PullRequest;

use crate::flag::Flag;
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
            Self::Reached(data) => data.orphans(linked),
            Self::Unreached(_) => Vec::new(),
        }
    }

    #[must_use]
    pub fn trunk(&self) -> Vec<Flag> {
        match self {
            Self::Reached(data) => data.trunk().iter().filter_map(Self::trunk_flag).collect(),
            Self::Unreached(_) => Vec::new(),
        }
    }

    fn trunk_flag(run: &TrunkRun) -> Option<Flag> {
        match run.verdict() {
            TrunkVerdict::Green => None,
            TrunkVerdict::Red => Some(Flag::TrunkRed(run.workflow().clone())),
            TrunkVerdict::Unknown => {
                let unseen = format!("no completed {} run on the default branch", run.workflow());
                Some(Flag::Unreached(GithubFailure::new(unseen)))
            }
        }
    }

    #[must_use]
    pub fn flag(&self) -> Option<Flag> {
        match self {
            Self::Reached(data) => data.truncated().map(|()| Flag::TruncatedOpen),
            Self::Unreached(failure) => Some(Flag::Unreached(failure.clone())),
        }
    }
}
