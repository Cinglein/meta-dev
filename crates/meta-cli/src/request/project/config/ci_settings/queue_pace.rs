use std::time::Duration;

use serde::Deserialize;

use super::millis::Millis;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Millis")]
pub struct QueuePace(Duration);

impl QueuePace {
    const FLOOR: Duration = Duration::from_secs(60);

    pub fn duration(self) -> Duration {
        self.0
    }
}

impl TryFrom<Millis> for QueuePace {
    type Error = String;

    fn try_from(millis: Millis) -> Result<Self, Self::Error> {
        let pace = millis.duration();
        (pace >= Self::FLOOR).then_some(Self(pace)).ok_or_else(|| {
            format!("queue_poll_millis must be at least {} ms: GitHub rate limits", Self::FLOOR.as_millis())
        })
    }
}
