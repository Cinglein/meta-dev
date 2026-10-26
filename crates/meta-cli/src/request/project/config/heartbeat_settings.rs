mod raw_heartbeat_settings;
mod seconds;

use std::time::Duration;

use serde::Deserialize;

use raw_heartbeat_settings::RawHeartbeatSettings;
pub use seconds::Seconds;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawHeartbeatSettings")]
pub struct HeartbeatSettings {
    interval: Seconds,
    tick: Seconds,
    stale: Seconds,
}

impl HeartbeatSettings {
    pub const DEFAULTS: &'static str = include_str!("heartbeat_settings/heartbeat_defaults.toml");

    pub fn interval(&self) -> Seconds {
        self.interval
    }

    pub fn tick_limit(&self) -> Duration {
        self.tick.duration()
    }

    pub fn stale_limit(&self) -> Duration {
        self.stale.duration()
    }
}

impl TryFrom<RawHeartbeatSettings> for HeartbeatSettings {
    type Error = &'static str;

    fn try_from(raw: RawHeartbeatSettings) -> Result<Self, Self::Error> {
        let ordered = raw.tick < raw.interval && raw.interval < raw.stale;
        ordered
            .then_some(Self { interval: raw.interval, tick: raw.tick, stale: raw.stale })
            .ok_or("heartbeat needs tick_seconds < interval_seconds < stale_seconds")
    }
}
