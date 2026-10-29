mod base_branch;
mod checks;
mod head_branch;
mod head_commit;
mod merge_state;
mod origin;
mod pull_state;
mod rollup;
mod rule_source;
mod ruled_pull;

use meta_schema::{Id, PullRequest, Task};
use serde::Deserialize;

pub use base_branch::BaseBranch;
use head_branch::HeadBranch;
pub use head_commit::HeadCommit;
pub use merge_state::MergeState;
use origin::Origin;
pub use pull_state::PullState;
use rollup::Rollup;
pub use rule_source::RuleSource;
pub use ruled_pull::RuledPull;

#[derive(Clone, Debug, Deserialize)]
pub struct LivePull {
    number: PullRequest,
    state: PullState,
    #[serde(rename = "baseRefName")]
    base: BaseBranch,
    #[serde(default, rename = "headRefName")]
    head: HeadBranch,
    #[serde(default, rename = "isCrossRepository")]
    origin: Origin,
    #[serde(default, rename = "statusCheckRollup")]
    rollup: Option<Rollup>,
    #[serde(default, rename = "mergeStateStatus")]
    merge: MergeState,
    #[serde(default, rename = "headRefOid")]
    commit: HeadCommit,
}

impl LivePull {
    #[must_use]
    pub fn number(&self) -> PullRequest {
        self.number
    }

    #[must_use]
    pub fn task(&self) -> Option<Id<Task>> {
        self.origin.same().and(self.head.task())
    }

    #[must_use]
    pub fn open_base(&self) -> Option<&BaseBranch> {
        (self.state == PullState::Open).then_some(&self.base)
    }

    #[must_use]
    pub fn base(&self) -> &BaseBranch {
        &self.base
    }
}
