mod sight;
mod unseen_run;

use meta_pull_query::{LiveData, TrunkRun, TrunkVerdict};
use meta_schema::PullRequest;
use meta_state::TaskState;

use crate::flag::Flag;
use crate::github_failure::GithubFailure;
use sight::Sight;
use unseen_run::UnseenRun;

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
    pub fn linked(&self, task: &TaskState) -> Vec<PullRequest> {
        let known = task.pull_requests();
        let headed = match self {
            Self::Reached(data) => data.headed(task.new_task().task()),
            Self::Unreached(_) => Vec::new(),
        };
        known.iter().copied().chain(headed.into_iter().filter(|pull_request| !known.contains(pull_request))).collect()
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
            TrunkVerdict::Unknown => Some(Flag::Unreached(GithubFailure::new(UnseenRun::new(run.workflow())))),
        }
    }

    #[must_use]
    pub fn finished(&self, linked: &[PullRequest]) -> Option<()> {
        linked.first()?;
        linked.iter().all(|pull_request| self.sight(*pull_request).ended().is_some()).then_some(())
    }

    #[must_use]
    pub fn flag(&self) -> Option<Flag> {
        match self {
            Self::Reached(data) => data.truncated().map(|()| Flag::TruncatedOpen),
            Self::Unreached(failure) => Some(Flag::Unreached(failure.clone())),
        }
    }
}
