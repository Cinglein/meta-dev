use std::fmt;
use std::str::FromStr;

use crate::request::ci::ci_status::CiStatus;
use crate::request::ci::commit::Commit;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchState {
    commit: Commit,
    status: CiStatus,
}

impl WatchState {
    pub fn new(commit: Commit, status: CiStatus) -> Self {
        Self { commit, status }
    }

    pub fn matching(&self, commit: &Commit) -> Option<&Self> {
        (self.commit == *commit).then_some(self)
    }

    pub fn commit(&self) -> &Commit {
        &self.commit
    }

    pub fn status(&self) -> CiStatus {
        self.status
    }
}

impl FromStr for WatchState {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (commit, status) = text.trim().split_once(' ').unwrap_or((text, ""));
        Ok(Self { commit: commit.parse()?, status: status.parse()? })
    }
}

impl fmt::Display for WatchState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.commit, self.status)
    }
}
