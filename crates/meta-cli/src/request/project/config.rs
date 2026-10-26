mod branch_name;
mod ci_settings;
mod remote_name;

use serde::Deserialize;

use crate::request::cli_failure::{CliFailure, Setup};
pub use branch_name::BranchName;
pub use ci_settings::{CiSettings, LogWindow, ProgramName};
use remote_name::RemoteName;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    remote: Option<RemoteName>,
    #[serde(default)]
    protected: Vec<BranchName>,
    #[serde(default)]
    ci: toml::Table,
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

    pub fn protects(&self, branch: &BranchName) -> Option<&BranchName> {
        self.protected.iter().find(|protected| *protected == branch)
    }
}
