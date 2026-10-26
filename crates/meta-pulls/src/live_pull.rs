mod checks;
mod pull_state;
mod rollup;
mod verdict;

use std::fmt;

use serde::Deserialize;

use checks::Checks;
use pull_state::PullState;
use rollup::Rollup;

#[derive(Clone, Debug, Deserialize)]
pub struct LivePull {
    state: PullState,
    #[serde(default, rename = "statusCheckRollup")]
    rollup: Option<Rollup>,
}

impl LivePull {
    #[must_use]
    pub fn red(&self) -> Option<&Self> {
        (self.state == PullState::Open).then(|| self.checks().failing()).flatten().map(|_| self)
    }

    fn checks(&self) -> Checks {
        self.rollup.as_ref().map_or(Checks::Absent, Rollup::checks)
    }
}

impl fmt::Display for LivePull {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.state, self.checks())
    }
}
