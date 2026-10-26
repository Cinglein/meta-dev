mod gc_mode;
mod seconds;
mod skew_limit;
mod stale_limit;
mod tick_limit;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

use super::Config;
use crate::request::cli_failure::{CliFailure, Setup};
pub use gc_mode::GcMode;
pub use seconds::Seconds;
pub use skew_limit::SkewLimit;
pub use stale_limit::StaleLimit;
pub use tick_limit::TickLimit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeartbeatSettings {
    interval: Seconds,
    tick: TickLimit,
    stale: StaleLimit,
    skew: SkewLimit,
    gc: GcMode,
}

impl HeartbeatSettings {
    pub fn gc_mode(&self) -> GcMode {
        self.gc
    }

    const DEFAULTS: &'static str = include_str!("heartbeat_settings/heartbeat_defaults.toml");
    const UNORDERED: &'static str = "tick_seconds < interval_seconds < stale_seconds is required";

    pub fn interval(&self) -> Seconds {
        self.interval
    }

    pub fn tick_limit(&self) -> TickLimit {
        self.tick
    }

    pub fn stale_limit(&self) -> StaleLimit {
        self.stale
    }

    pub fn skew_limit(&self) -> SkewLimit {
        self.skew
    }
}

impl TryFrom<&Config> for HeartbeatSettings {
    type Error = CliFailure;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let mut merged: toml::Table = toml::from_str(Self::DEFAULTS).unwrap_or_default();
        merged.extend(config.heartbeat.clone());
        toml::Value::Table(merged)
            .try_into()
            .map_err(|failure| CliFailure::raised::<Setup>(format!("meta.toml [heartbeat]: {failure}")))
    }
}

impl<'de> Deserialize<'de> for HeartbeatSettings {
    fn deserialize<Source: Deserializer<'de>>(source: Source) -> Result<Self, Source::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields {
            #[serde(rename = "interval_seconds")]
            interval: Seconds,
            #[serde(rename = "tick_seconds")]
            tick: TickLimit,
            #[serde(rename = "stale_seconds")]
            stale: StaleLimit,
            #[serde(rename = "skew_seconds")]
            skew: SkewLimit,
            gc: GcMode,
        }
        let Fields { interval, tick, stale, skew, gc } = Fields::deserialize(source)?;
        tick.under(interval)
            .and(stale.over(interval))
            .map(|()| Self { interval, tick, stale, skew, gc })
            .ok_or_else(|| Source::Error::custom(Self::UNORDERED))
    }
}
