mod branch_name;
mod ci_settings;
mod heartbeat_settings;

use meta_log::RemoteName;
use meta_pull_query::WorkflowName;
use serde::Deserialize;

use crate::request::cli_failure::{CliFailure, Setup};
pub use branch_name::BranchName;
pub use ci_settings::{CiSettings, LogWindow, ProgramName};
pub use heartbeat_settings::{GcMode, HeartbeatSettings, TickLimit};

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    remote: Option<RemoteName>,
    #[serde(default)]
    protected: Vec<BranchName>,
    #[serde(default)]
    log_branches: Vec<BranchName>,
    #[serde(default)]
    required_workflows: Vec<WorkflowName>,
    #[serde(default)]
    ci: toml::Table,
    #[serde(default)]
    heartbeat: toml::Table,
}

impl Config {
    pub fn remote(&self) -> Result<&RemoteName, CliFailure> {
        self.remote.as_ref().ok_or_else(|| CliFailure::raised::<Setup>("meta.toml: `remote` is required"))
    }

    pub fn required_workflows(&self) -> &[WorkflowName] {
        &self.required_workflows
    }

    pub fn push_fix(&self, branch: &BranchName) -> &'static str {
        self.log_branches.iter().find(|log| *log == branch).map_or("open a PR", |_| "only `meta` writes it")
    }

    pub fn protects(&self, branch: &BranchName) -> Option<&BranchName> {
        self.protected.iter().find(|protected| *protected == branch)
    }
}
