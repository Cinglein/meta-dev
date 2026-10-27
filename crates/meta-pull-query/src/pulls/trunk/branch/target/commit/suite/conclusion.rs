use serde::Deserialize;

use crate::trunk_verdict::TrunkVerdict;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Conclusion {
    Success,
    Failure,
    TimedOut,
    StartupFailure,
    #[serde(other)]
    Undecided,
}

impl Conclusion {
    pub fn verdict(self) -> Option<TrunkVerdict> {
        match self {
            Self::Success => Some(TrunkVerdict::Green),
            Self::Failure | Self::TimedOut | Self::StartupFailure => Some(TrunkVerdict::Red),
            Self::Undecided => None,
        }
    }
}
