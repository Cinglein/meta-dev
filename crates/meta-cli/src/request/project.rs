mod config;

use std::path::{Path, PathBuf};
use std::process::Command;

use meta_log::GitLog;

use crate::request::ci::Commit;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::Listed;
pub use config::{BranchName, CiSettings, Config, LogWindow, ProgramName};
#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    config: Config,
}

impl Project {
    const CONFIG_FILE: &'static str = "meta.toml";
    const GIT_ENTRY: &'static str = ".git";
    pub fn log(&self) -> Result<GitLog, CliFailure> {
        self.config.remote().map(|remote| GitLog::new(self.root.clone(), remote.clone()))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn push_current_branch(&self) -> Option<Result<Commit, Listed>> {
        let remote = self.config.remote().ok()?.to_string();
        let unprotected = self.unprotected_branch()?;
        let pushed = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["push", "--quiet", "-u", &remote, &unprotected.to_string()])
            .output()
            .ok()?;
        let head = Command::new("git").arg("-C").arg(&self.root).args(["rev-parse", "HEAD"]).output().ok()?;
        Some(if pushed.status.success() {
            String::from_utf8_lossy(&head.stdout).parse().map_err(|never| match never {})
        } else {
            Err(Listed::new(format!(
                "push of {unprotected} failed: {}",
                String::from_utf8_lossy(&pushed.stderr).trim()
            )))
        })
    }

    pub fn pushed_head(&self) -> Option<Commit> {
        let tracking = format!("refs/remotes/{}/{}", self.config.remote().ok()?, self.unprotected_branch()?);
        let head = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["rev-parse", "--verify", "-q", &tracking])
            .output()
            .ok()?;
        let Ok(commit) = String::from_utf8_lossy(&head.stdout).parse();
        head.status.success().then_some(commit)
    }

    fn unprotected_branch(&self) -> Option<BranchName> {
        let current = Command::new("git").arg("-C").arg(&self.root).args(["branch", "--show-current"]).output().ok()?;
        let branch: BranchName = String::from_utf8_lossy(&current.stdout).trim().parse().ok()?;
        Some(branch).filter(|branch| !branch.to_string().is_empty() && self.config.protects(branch).is_none())
    }
}

impl TryFrom<&Path> for Project {
    type Error = CliFailure;

    fn try_from(dir: &Path) -> Result<Self, Self::Error> {
        let root = dir
            .ancestors()
            .find(|ancestor| ancestor.join(Self::GIT_ENTRY).exists())
            .map(Path::to_path_buf)
            .ok_or_else(|| CliFailure::raised::<Setup>("not inside a git repository"))?;
        let path = root.join(Self::CONFIG_FILE);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| CliFailure::raised::<Setup>(format!("{}: {e}", path.display())))?;
        let config =
            toml::from_str(&text).map_err(|e| CliFailure::raised::<Setup>(format!("{}: {e}", path.display())))?;
        Ok(Self { root, config })
    }
}
