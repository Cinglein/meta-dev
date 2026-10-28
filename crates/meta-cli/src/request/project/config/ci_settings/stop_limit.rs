use std::time::{Duration, Instant};

use serde::Deserialize;

use super::millis::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Millis")]
pub struct StopLimit(Duration);

impl StopLimit {
    const HOOK_TIMEOUT: Duration = Duration::from_secs(10);
    const MARGIN: Duration = Duration::from_secs(3);
    const CEILING: Duration = Self::HOOK_TIMEOUT.saturating_sub(Self::MARGIN);

    pub fn deadline(self) -> Instant {
        Instant::now() + self.0
    }
}

impl TryFrom<Millis> for StopLimit {
    type Error = String;

    fn try_from(millis: Millis) -> Result<Self, Self::Error> {
        let limit = millis.duration();
        (limit < Self::CEILING).then_some(Self(limit)).ok_or_else(|| {
            format!(
                "stop_millis must be under {} ms: Claude Code ends a Stop hook at {} s",
                Self::CEILING.as_millis(),
                Self::HOOK_TIMEOUT.as_secs()
            )
        })
    }
}
