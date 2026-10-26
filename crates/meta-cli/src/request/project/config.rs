mod branch_name;
mod ci_settings;
mod heartbeat_settings;

use meta_log::RemoteName;
use serde::Deserialize;

use crate::request::cli_failure::{CliFailure, Setup};
pub use branch_name::BranchName;
pub use ci_settings::{CiSettings, LogWindow, ProgramName};
pub use heartbeat_settings::{HeartbeatSettings, Seconds};

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    remote: Option<RemoteName>,
    #[serde(default)]
    protected: Vec<BranchName>,
    #[serde(default)]
    log_branches: Vec<BranchName>,
    #[serde(default)]
    ci: toml::Table,
    #[serde(default)]
    heartbeat: toml::Table,
}

impl Config {
    pub fn remote(&self) -> Result<&RemoteName, CliFailure> {
        self.remote.as_ref().ok_or_else(|| CliFailure::raised::<Setup>("meta.toml: `remote` is required"))
    }

    pub fn ci_settings(&self) -> CiSettings {
        let mut merged: toml::Table = toml::from_str(CiSettings::DEFAULTS).unwrap_or_default();
        merged.extend(self.ci.clone());
        toml::Value::Table(merged).try_into().unwrap_or_default()
    }

    pub fn heartbeat_settings(&self) -> Result<HeartbeatSettings, CliFailure> {
        let mut merged: toml::Table = toml::from_str(HeartbeatSettings::DEFAULTS).unwrap_or_default();
        merged.extend(self.heartbeat.clone());
        toml::Value::Table(merged)
            .try_into()
            .map_err(|failure| CliFailure::raised::<Setup>(format!("meta.toml [heartbeat]: {failure}")))
    }

    pub fn push_fix(&self, branch: &BranchName) -> &'static str {
        self.log_branches.iter().find(|log| *log == branch).map_or("open a PR", |_| "only `meta` writes it")
    }

    pub fn protects(&self, branch: &BranchName) -> Option<&BranchName> {
        self.protected.iter().find(|protected| *protected == branch)
    }
}
