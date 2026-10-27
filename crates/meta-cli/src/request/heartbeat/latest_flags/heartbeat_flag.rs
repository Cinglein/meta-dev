use std::fmt;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeartbeatFlag {
    NeverTicked,
    Stale { age: Duration, limit: Duration },
    ClockSkew { ahead: Duration, limit: Duration },
    GcDisabled,
}

impl fmt::Display for HeartbeatFlag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NeverTicked => writeln!(formatter, "heartbeat stale: never ticked"),
            Self::Stale { age, limit } => writeln!(
                formatter,
                "heartbeat stale: {}s since the last tick, limit {}s",
                age.as_secs(),
                limit.as_secs()
            ),
            Self::ClockSkew { ahead, limit } => writeln!(
                formatter,
                "heartbeat clock skew: snapshot is {}s in the future, limit {}s",
                ahead.as_secs(),
                limit.as_secs()
            ),
            Self::GcDisabled => {
                writeln!(formatter, "heartbeat gc disabled: [heartbeat] gc must be \"dry-run\" or \"free\"")
            }
        }
    }
}
