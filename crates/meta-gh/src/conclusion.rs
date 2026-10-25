use std::fmt;

use serde::Deserialize;

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
    pub fn failing(self) -> Option<Self> {
        match self {
            Self::Failure | Self::Cancelled | Self::TimedOut | Self::StartupFailure | Self::ActionRequired => {
                Some(self)
            }
            Self::Success | Self::Skipped | Self::Neutral | Self::Stale | Self::Unknown => None,
        }
    }

    pub fn succeeded(self) -> Option<Self> {
        match self {
            Self::Success | Self::Skipped | Self::Neutral => Some(self),
            Self::Failure
            | Self::Cancelled
            | Self::TimedOut
            | Self::StartupFailure
            | Self::ActionRequired
            | Self::Stale
            | Self::Unknown => None,
        }
    }
}

impl fmt::Display for Conclusion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
