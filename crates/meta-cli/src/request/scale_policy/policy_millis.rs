mod pace_floor;

use std::time::Duration;

use serde::Deserialize;

use crate::request::cli_failure::{CliFailure, Setup};
use pace_floor::PaceFloor;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct PolicyMillis {
    #[serde(default)]
    queue_pace: Option<PaceFloor>,
}

impl PolicyMillis {
    const MISSING: &'static str = "tooling/policy.toml: [millis] queue_pace is required";

    pub fn queue_floor(self) -> Result<Duration, CliFailure> {
        self.queue_pace.map(PaceFloor::duration).ok_or_else(|| CliFailure::raised::<Setup>(Self::MISSING))
    }
}
