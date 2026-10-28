mod base_branch;
mod checks;
mod pull_state;
mod rollup;
mod rule_source;
mod ruled_pull;

use meta_schema::PullRequest;
use serde::Deserialize;

pub use base_branch::BaseBranch;
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
    #[serde(default, rename = "statusCheckRollup")]
    rollup: Option<Rollup>,
}

impl LivePull {
    #[must_use]
    pub fn number(&self) -> PullRequest {
        self.number
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
