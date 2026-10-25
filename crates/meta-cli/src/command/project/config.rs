mod branch_name;
mod ci_settings;
mod remote_name;

use serde::Deserialize;

use crate::command::cli_failure::CliFailure;
pub use branch_name::BranchName;
pub use ci_settings::{CiSettings, ProgramName};
use remote_name::RemoteName;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    remote: Option<RemoteName>,
    #[serde(default)]
    protected: Vec<BranchName>,
    #[serde(default = "CiSettings::embedded")]
    ci: CiSettings,
}

impl Config {
    pub fn remote(&self) -> Result<&RemoteName, CliFailure> {
        self.remote.as_ref().ok_or_else(|| CliFailure::config("meta.toml: `remote` is required"))
    }

    pub fn ci_settings(&self) -> &CiSettings {
        &self.ci
    }

    pub fn protects(&self, branch: &BranchName) -> Option<&BranchName> {
        self.protected.iter().find(|protected| *protected == branch)
    }
}
