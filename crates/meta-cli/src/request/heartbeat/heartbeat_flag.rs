use std::fmt;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeartbeatFlag {
    NeverTicked,
    Stale { age: Duration, limit: Duration },
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
        }
    }
}
