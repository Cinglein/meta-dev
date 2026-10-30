use std::fmt;

use chrono::TimeDelta;
use meta_schema::CommitId;

use super::head_commit::HeadCommit;
use super::merge_state::MergeState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unrun {
    commit: HeadCommit,
    state: MergeState,
    within: TimeDelta,
}

impl Unrun {
    #[must_use]
    pub fn new(commit: HeadCommit, state: MergeState, within: TimeDelta) -> Self {
        Self { commit, state, within }
    }

    #[must_use]
    pub fn waited_at(self, head: Option<&CommitId>) -> Option<Self> {
        head.is_none_or(|head| head.to_string() == self.commit.to_string()).then_some(self)
    }
}

impl fmt::Display for Unrun {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "no CI run for {} after {} min; GitHub merge state={}",
            self.commit,
            self.within.num_minutes(),
            self.state
        )
    }
}
