mod remote_name;

use serde::Deserialize;

use crate::command::cli_failure::CliFailure;
use remote_name::RemoteName;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    remote: Option<RemoteName>,
}

impl Config {
    pub fn remote(&self) -> Result<&RemoteName, CliFailure> {
        self.remote.as_ref().ok_or_else(|| CliFailure::config("meta.toml: `remote` is required"))
    }
}
