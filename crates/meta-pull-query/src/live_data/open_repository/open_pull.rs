use meta_schema::PullRequest;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct OpenPull {
    number: PullRequest,
}

impl OpenPull {
    pub fn unlinked(self, linked: &[PullRequest]) -> Option<PullRequest> {
        (!linked.contains(&self.number)).then_some(self.number)
    }
}
