use meta_pulls::{LivePull, Page};
use meta_schema::{Id, PullRequest, Task};
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
    pub fn headed(&self, task: Id<Task>) -> Vec<PullRequest> {
        let named = self.pull_requests.nodes().filter(|pull| pull.task() == Some(task));
        named.map(LivePull::number).collect()
    }

    #[must_use]
    pub fn continued(self, next: Self) -> Self {
        Self { pull_requests: self.pull_requests.continued(next.pull_requests) }
    }

    #[must_use]
    pub fn orphans(&self, linked: &[PullRequest]) -> Vec<PullRequest> {
        self.pull_requests.nodes().map(LivePull::number).filter(|number| !linked.contains(number)).collect()
    }
}
