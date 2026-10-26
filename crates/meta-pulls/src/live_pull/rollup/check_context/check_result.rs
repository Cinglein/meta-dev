use serde::Deserialize;

use crate::verdict::Verdict;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckResult {
    Success,
    Neutral,
    Skipped,
    Failure,
    Error,
    Cancelled,
    TimedOut,
    ActionRequired,
    StartupFailure,
    Pending,
    Expected,
    Stale,
    #[serde(other)]
    Unknown,
}

impl CheckResult {
    pub fn verdict(self) -> Verdict {
        match self {
            Self::Success | Self::Neutral | Self::Skipped => Verdict::Green,
            Self::Failure
            | Self::Error
            | Self::Cancelled
            | Self::TimedOut
            | Self::ActionRequired
            | Self::StartupFailure => Verdict::Red,
            Self::Pending | Self::Expected | Self::Stale | Self::Unknown => Verdict::Pending,
        }
    }
}
