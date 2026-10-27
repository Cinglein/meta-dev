mod agent_root;
mod cause;
mod kept_entries;
mod limit;
mod spared_name;
mod tick_count;
mod watch_share;

#[cfg(test)]
mod tests;

use std::path::Path;
use std::time::{Duration, Instant};

use chrono::{DateTime, TimeDelta, Utc};
use serde::Deserialize;

use crate::request::project::HeartbeatSettings;
use serde::de::Error as _;

pub use agent_root::AgentRoot;
pub use cause::Cause;
pub use kept_entries::KeptEntries;
pub use limit::Limit;
pub use spared_name::SparedName;
pub use tick_count::TickCount;
use watch_share::WatchShare;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorSettings {
    #[serde(rename = "scratch")]
    roots: Vec<AgentRoot>,
    spared: Vec<SparedName>,
    stop: Vec<Cause>,
    #[serde(rename = "orphan_seconds")]
    orphan: Limit,
    #[serde(rename = "cpu_seconds")]
    cpu: Limit,
    #[serde(rename = "wall_seconds")]
    wall: Limit,
    #[serde(rename = "grace_seconds")]
    grace: Limit,
    #[serde(rename = "probe_seconds")]
    probe: Limit,
    #[serde(rename = "recent_seconds")]
    recent: Limit,
    #[serde(rename = "kept_entries")]
    kept: KeptEntries,
    #[serde(rename = "watch_percent")]
    watch: WatchShare,
    #[serde(rename = "watch_ticks")]
    promote: TickCount,
    #[serde(rename = "mark_seconds")]
    mark: Limit,
    #[serde(rename = "start_slack_seconds")]
    slack: Limit,
}

impl MonitorSettings {
    const DEFAULTS: &'static str = include_str!("monitor_settings/monitor_defaults.toml");
    const GRACE_CEILING: Duration = Duration::from_secs(30);
    const PROBE_CEILING: Duration = Duration::from_secs(30);
    const SLACK_CEILING: Duration = Duration::from_secs(30);
    const TOO_WIDE: &'static str =
        "worktree root and scratch dirs may not be empty, `.`, `/`, or hold $HOME or this clone";
    const TOO_LONG: &'static str =
        "grace_seconds, probe_seconds and start_slack_seconds are at most 30, watch_percent at most 100";

    pub fn merged(
        overrides: toml::Table,
        checkout: &Path,
        worktrees: &Path,
        home: &Path,
    ) -> Result<Self, toml::de::Error> {
        let mut merged: toml::Table = toml::from_str(Self::DEFAULTS).unwrap_or_default();
        merged.extend(overrides);
        let read: Self = toml::Value::Table(merged).try_into()?;
        let declared = [AgentRoot::own(worktrees)].into_iter().chain(read.roots);
        let declared: Option<Vec<AgentRoot>> = declared.map(|root| root.anchored(checkout, home)).collect();
        let declared = declared.ok_or_else(|| toml::de::Error::custom(Self::TOO_WIDE))?;
        (read.grace.duration() <= Self::GRACE_CEILING && read.probe.duration() <= Self::PROBE_CEILING)
            .then_some(())
            .filter(|()| read.slack.duration() <= Self::SLACK_CEILING)
            .and_then(|()| read.watch.bounded())
            .ok_or_else(|| toml::de::Error::custom(Self::TOO_LONG))?;
        let roots = [AgentRoot::own(checkout)].into_iter().chain(declared).collect();
        Ok(Self { roots, spared: SparedName::floor().into_iter().chain(read.spared).collect(), ..read })
    }

    pub fn fallback() -> Option<Self> {
        toml::from_str(Self::DEFAULTS).ok()
    }

    pub fn mark_fresh(&self, seen: DateTime<Utc>, now: DateTime<Utc>) -> Option<()> {
        let idle = (now - seen).to_std().unwrap_or_default();
        self.mark.passed_by(idle).map_or(Some(()), |()| None)
    }

    pub fn start_slack(&self) -> TimeDelta {
        TimeDelta::from_std(self.slack.duration()).unwrap_or_default()
    }

    pub fn paced(&self, heartbeat: &HeartbeatSettings) -> Option<()> {
        self.mark.passed_by(heartbeat.interval().duration()).map_or(Some(()), |()| None)
    }

    pub fn roots(&self) -> &[AgentRoot] {
        &self.roots
    }

    pub fn spared(&self) -> &[SparedName] {
        &self.spared
    }

    pub fn stops(&self, cause: Cause) -> Option<()> {
        self.stop.contains(&cause).then_some(())
    }

    pub fn cause(&self, age: Duration, cpu_time: Duration, orphaned: Option<()>) -> Option<Cause> {
        let orphaned = orphaned.and_then(|()| self.orphan.passed_by(age)).map(|()| Cause::Orphaned);
        orphaned
            .or_else(|| self.cpu.passed_by(cpu_time).map(|()| Cause::CpuTime))
            .or_else(|| self.wall.passed_by(age).map(|()| Cause::WallTime))
    }

    pub fn grace(&self) -> Limit {
        self.grace
    }

    pub fn probe_deadline(&self, from: Instant) -> Instant {
        from + self.probe.duration()
    }

    pub fn shows(&self, age: Duration) -> Option<()> {
        self.recent.passed_by(age).map_or(Some(()), |()| None)
    }

    pub fn kept(&self) -> KeptEntries {
        self.kept
    }

    pub fn near(&self, age: Duration, cpu_time: Duration) -> Option<()> {
        self.watch.reached(cpu_time, self.cpu).or_else(|| self.watch.reached(age, self.wall))
    }

    pub fn promotes(&self, ticks: TickCount) -> Option<()> {
        (ticks >= self.promote).then_some(())
    }
}
