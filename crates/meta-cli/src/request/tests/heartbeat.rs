mod flags;
mod gc;
mod plist;
mod scene;
mod settings;
mod skew;
mod staging;
mod supervised;
mod tick;

use chrono::{DateTime, TimeDelta, Utc};

use crate::request::cli_failure::CliFailure;
use crate::request::project::{Config, HeartbeatSettings};
use scene::Scene;

const START: &str = "2026-10-26T12:00:00Z";
const SETTINGS: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 60\nstale_seconds = 900";

struct Beat;

impl Beat {
    pub fn after(offset: TimeDelta) -> DateTime<Utc> {
        START.parse::<DateTime<Utc>>().unwrap_or_default() + offset
    }

    pub fn settings(text: &str) -> Result<HeartbeatSettings, CliFailure> {
        HeartbeatSettings::try_from(&toml::from_str::<Config>(text).unwrap_or_default())
    }
}
