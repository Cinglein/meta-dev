mod open_pull;

use meta_pulls::Nodes;
use meta_schema::PullRequest;
use serde::Deserialize;

use open_pull::OpenPull;

#[derive(Clone, Debug, Deserialize)]
pub struct OpenRepository {
    #[serde(rename = "pullRequests")]
    pull_requests: Nodes<OpenPull>,
}

impl OpenRepository {
    #[must_use]
    pub fn orphans(&self, linked: &[PullRequest]) -> Vec<PullRequest> {
        self.pull_requests.iter().copied().filter_map(|pull| pull.unlinked(linked)).collect()
    }
}
