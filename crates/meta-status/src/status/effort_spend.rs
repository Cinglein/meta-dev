use std::fmt;

use meta_state::TaskState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffortSpend {
    Unmeasured,
}

impl EffortSpend {
    #[must_use]
    pub fn measured_for(_task: &TaskState) -> Self {
        Self::Unmeasured
    }
}

impl fmt::Display for EffortSpend {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unmeasured => formatter.write_str("spend unmeasured"),
        }
    }
}
