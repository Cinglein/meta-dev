mod open_repository;

use std::collections::HashMap;

use meta_schema::PullRequest;
use serde::Deserialize;

use meta_pulls::LivePull;

use crate::pull_alias::PullAlias;
use open_repository::OpenRepository;

#[derive(Clone, Debug, Deserialize)]
pub struct LiveData {
    open: OpenRepository,
    #[serde(default)]
    linked: HashMap<PullAlias, Option<LivePull>>,
}

impl LiveData {
    #[must_use]
    pub fn pull(&self, pull_request: PullRequest) -> Option<&LivePull> {
        self.linked.get(&PullAlias::new(pull_request)).and_then(Option::as_ref)
    }

    #[must_use]
    pub fn open(&self) -> &OpenRepository {
        &self.open
    }
}
