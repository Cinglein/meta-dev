use meta_pulls::{LivePull, Page};
use meta_schema::PullRequest;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct OpenRepository {
    #[serde(rename = "pullRequests")]
    pull_requests: Page<LivePull>,
}

impl OpenRepository {
    pub fn pull_requests(&self) -> &Page<LivePull> {
        &self.pull_requests
    }

    #[must_use]
    pub fn orphans(&self, linked: &[PullRequest]) -> Vec<PullRequest> {
        self.pull_requests.nodes().map(LivePull::number).filter(|number| !linked.contains(number)).collect()
    }
}
