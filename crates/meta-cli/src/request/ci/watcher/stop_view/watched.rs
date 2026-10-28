use crate::request::ci::Commit;
use crate::request::ci::ci_status::CiStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Watched {
    commit: Commit,
    status: Option<CiStatus>,
}

impl Watched {
    pub fn new(commit: Commit, status: Option<CiStatus>) -> Self {
        Self { commit, status }
    }

    pub fn commit(&self) -> &Commit {
        &self.commit
    }

    pub fn status(&self) -> Option<CiStatus> {
        self.status
    }

    pub fn restarted(&self) -> Option<&Commit> {
        self.status.and_then(CiStatus::finished).map_or(Some(&self.commit), |_| None)
    }
}
