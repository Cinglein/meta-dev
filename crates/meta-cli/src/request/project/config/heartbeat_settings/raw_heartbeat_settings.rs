use serde::Deserialize;

use super::seconds::Seconds;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHeartbeatSettings {
    #[serde(rename = "interval_seconds")]
    pub interval: Seconds,
    #[serde(rename = "tick_seconds")]
    pub tick: Seconds,
    #[serde(rename = "stale_seconds")]
    pub stale: Seconds,
}
