mod head_ref;

use serde::Deserialize;

use super::head_commit::HeadCommit;
use super::merge_state::MergeState;
use head_ref::HeadRef;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Merging {
    #[serde(rename = "mergeStateStatus")]
    state: MergeState,
    #[serde(rename = "headRefOid")]
    commit: HeadCommit,
    #[serde(default, rename = "headRef")]
    head_ref: Option<HeadRef>,
}

impl Merging {
    pub fn state(&self) -> MergeState {
        self.state
    }

    pub fn commit(&self) -> &HeadCommit {
        &self.commit
    }

    pub fn head_ref(&self) -> Option<&HeadRef> {
        self.head_ref.as_ref()
    }
}
