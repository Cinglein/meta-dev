mod base_branch;
mod checks;
mod head_branch;
mod pull_state;
mod rollup;
mod ruled_pull;

use meta_schema::PullRequest;
use serde::Deserialize;

pub use base_branch::BaseBranch;
pub use head_branch::HeadBranch;
pub use pull_state::PullState;
use rollup::Rollup;
pub use ruled_pull::RuledPull;

#[derive(Clone, Debug, Deserialize)]
pub struct LivePull {
    number: PullRequest,
    state: PullState,
    #[serde(rename = "baseRefName")]
    base: BaseBranch,
    #[serde(default, rename = "headRefName")]
    head: HeadBranch,
    #[serde(default, rename = "statusCheckRollup")]
    rollup: Option<Rollup>,
}

impl LivePull {
    #[must_use]
    pub fn number(&self) -> PullRequest {
        self.number
    }

    #[must_use]
    pub fn head(&self) -> &HeadBranch {
        &self.head
    }

    #[must_use]
    pub fn base(&self) -> &BaseBranch {
        &self.base
    }
}
