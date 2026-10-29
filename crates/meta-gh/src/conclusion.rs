use serde::Deserialize;

use crate::outcome::Outcome;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
    #[must_use]
    pub fn outcome(self) -> Outcome {
        match self {
            Self::Success => Outcome::Succeeded,
            Self::Failure | Self::Cancelled | Self::TimedOut | Self::StartupFailure | Self::ActionRequired => {
                Outcome::Failing
            }
            Self::Skipped | Self::Neutral | Self::Stale | Self::Unknown => Outcome::Undecided,
        }
    }

    #[must_use]
    pub fn behind_a_newer_run(self) -> Option<Self> {
        match self {
            Self::Cancelled => None,
            Self::Success
            | Self::Failure
            | Self::Skipped
            | Self::TimedOut
            | Self::ActionRequired
            | Self::Neutral
            | Self::Stale
            | Self::StartupFailure
            | Self::Unknown => Some(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Conclusion;
    use crate::outcome::Outcome;

    const CANCELLED: &str = r#""CANCELLED""#;

    #[test]
    fn a_cancelled_run_is_failing_so_ci_gating_fails_closed_on_work_it_never_checked() {
        let cancelled = serde_json::from_str::<Conclusion>(CANCELLED).unwrap();
        assert_eq!(cancelled.outcome(), Outcome::Failing);
    }
}
