use serde::Deserialize;

use crate::outcome::Outcome;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Conclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
    Neutral,
    Stale,
    StartupFailure,
    #[serde(other)]
    Unknown,
}

impl Conclusion {
    pub fn outcome(self) -> Outcome {
        match self {
            Self::Success | Self::Skipped | Self::Neutral => Outcome::Succeeded,
            Self::Failure | Self::Cancelled | Self::TimedOut | Self::StartupFailure | Self::ActionRequired => {
                Outcome::Failing
            }
            Self::Stale | Self::Unknown => Outcome::Undecided,
        }
    }
}
