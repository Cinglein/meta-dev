use serde::Deserialize;

use super::head_commit::HeadCommit;
use super::merge_state::MergeState;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Merging {
    #[serde(rename = "mergeStateStatus")]
    state: MergeState,
    #[serde(rename = "headRefOid")]
    commit: HeadCommit,
}

impl Merging {
    pub fn state(&self) -> MergeState {
        self.state
    }

    pub fn commit(&self) -> &HeadCommit {
        &self.commit
    }
}
