mod config;

use std::path::{Path, PathBuf};
use std::process::Command;

use meta_log::GitLog;

use crate::command::cli_failure::CliFailure;
use crate::command::output::Listed;
pub use config::BranchName;
use config::Config;

#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    config: Config,
}

impl Project {
    const CONFIG_FILE: &'static str = "meta.toml";

    pub fn discover(dir: &Path) -> Result<Self, CliFailure> {
        let output = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(CliFailure::config)?;
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

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn push_current_branch(&self) -> Option<Listed> {
        let remote = self.config.remote().ok()?.to_string();
        let current = Command::new("git").arg("-C").arg(&self.root).args(["branch", "--show-current"]).output().ok()?;
        let branch: BranchName = String::from_utf8_lossy(&current.stdout).trim().parse().ok()?;
        let unprotected =
            Some(branch).filter(|branch| !branch.to_string().is_empty() && self.config.protects(branch).is_none())?;
        let pushed = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["push", "--quiet", "-u", &remote, &unprotected.to_string()])
            .output()
            .ok()?;
        Some(if pushed.status.success() {
            Listed::new(format!("pushed {unprotected} to {remote}"))
        } else {
            Listed::new(format!("push of {unprotected} failed: {}", String::from_utf8_lossy(&pushed.stderr).trim()))
        })
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.root.display())
    }
}
