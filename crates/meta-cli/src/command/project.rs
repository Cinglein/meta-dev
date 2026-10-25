mod config;

use std::path::PathBuf;
use std::process::Command;

use meta_log::GitLog;

use crate::command::cli_failure::CliFailure;
use config::Config;

#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    config: Config,
}

impl Project {
    const CONFIG_FILE: &'static str = "meta.toml";

    pub fn discover() -> Result<Self, CliFailure> {
        let output = Command::new("git").args(["rev-parse", "--show-toplevel"]).output().map_err(CliFailure::config)?;
        let root = output
            .status
            .success()
            .then(|| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()))
            .ok_or_else(|| CliFailure::config("not inside a git repository"))?;
        let path = root.join(Self::CONFIG_FILE);
        let text =
            std::fs::read_to_string(&path).map_err(|e| CliFailure::config(format!("{}: {e}", path.display())))?;
        let config = toml::from_str(&text).map_err(|e| CliFailure::config(format!("{}: {e}", path.display())))?;
        Ok(Self { root, config })
    }

    pub fn log(&self) -> Result<GitLog, CliFailure> {
        self.config.remote().map(|remote| GitLog::new(self.root.clone(), remote))
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.root.display())
    }
}
