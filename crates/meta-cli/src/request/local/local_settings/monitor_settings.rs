mod agent_root;
mod cause;
mod core_share;
mod limit;
mod spared_name;

use std::num::NonZeroUsize;
use std::path::Path;
use std::time::{Duration, Instant};

use serde::Deserialize;

pub use agent_root::AgentRoot;
pub use cause::Cause;
pub use core_share::CoreShare;
pub use limit::Limit;
pub use spared_name::SparedName;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorSettings {
    roots: Vec<AgentRoot>,
    spared: Vec<SparedName>,
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
    #[serde(rename = "load_percent_of_cores")]
    load: CoreShare,
    #[serde(rename = "agent_cpu_percent")]
    agent_cpu: CoreShare,
}

impl MonitorSettings {
    const DEFAULTS: &'static str = include_str!("monitor_settings/monitor_defaults.toml");

    pub fn merged(overrides: toml::Table, checkout: &Path) -> Result<Self, toml::de::Error> {
        let mut merged: toml::Table = toml::from_str(Self::DEFAULTS).unwrap_or_default();
        merged.extend(overrides);
        let read: Self = toml::Value::Table(merged).try_into()?;
        Ok(Self { roots: read.roots.into_iter().map(|root| root.anchored(checkout)).collect(), ..read })
    }

    pub fn roots(&self) -> &[AgentRoot] {
        &self.roots
    }

    pub fn spared(&self) -> &[SparedName] {
        &self.spared
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

    pub fn overloaded(&self, seen: CoreShare, cores: NonZeroUsize) -> Option<CoreShare> {
        self.load.scaled(cores).passed_by(seen)
    }

    pub fn agent_cpu(&self) -> CoreShare {
        self.agent_cpu
    }
}
