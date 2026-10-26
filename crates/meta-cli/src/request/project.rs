mod config;

use std::path::{Path, PathBuf};
use std::process::Command;

use meta_log::GitLog;

use crate::request::ci::{Commit, ReflogEntry, UnixSeconds};
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

    pub fn created_since(&self, start: UnixSeconds) -> Vec<Commit> {
        let walked = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["log", "--walk-reflogs", "--date=unix", ReflogEntry::FORMAT, "HEAD", "--"])
            .output();
        let text = walked.map(|walked| String::from_utf8_lossy(&walked.stdout).into_owned()).unwrap_or_default();
        let created: Vec<Commit> = text
            .lines()
            .filter_map(|line| line.parse::<ReflogEntry>().ok())
            .filter_map(|entry| entry.created_since(start).cloned())
            .collect();
        created
            .iter()
            .enumerate()
            .filter(|(at, commit)| created.iter().position(|other| other == *commit) == Some(*at))
            .map(|(_, commit)| commit.clone())
            .collect()
    }

    pub fn pushed_tips(&self, commits: &[Commit]) -> Vec<Commit> {
        let remote = commits.first().and_then(|_| self.config.remote().ok());
        let unpushed = remote.and_then(|remote| self.unpushed_of(commits, &remote.to_string()));
        let pushed: Vec<Commit> = unpushed.map_or_else(Vec::new, |unpushed| {
            commits.iter().filter(|commit| !unpushed.contains(commit)).cloned().collect()
        });
        pushed.get(1).and_then(|_| self.independent(&pushed)).unwrap_or(pushed)
    }

    pub fn unpushed(&self) -> Option<Commit> {
        let remotes = format!("--remotes={}", self.config.remote().ok()?);
        let ahead = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["rev-list", "-1", "HEAD", "--not", &remotes])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&ahead.stdout);
        let Ok(commit) = text.parse();
        (ahead.status.success() && !text.trim().is_empty()).then_some(commit)
    }

    fn independent(&self, commits: &[Commit]) -> Option<Vec<Commit>> {
        let tips = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["merge-base", "--independent"])
            .args(commits.iter().map(Commit::argument))
            .output()
            .ok()
            .filter(|tips| tips.status.success())?;
        let Ok(tips) = String::from_utf8_lossy(&tips.stdout).lines().map(str::parse).collect();
        Some(tips)
    }

    fn unpushed_of(&self, commits: &[Commit], remote: &str) -> Option<Vec<Commit>> {
        let ahead = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["rev-list", "--ignore-missing"])
            .args(commits.iter().map(Commit::argument))
            .args(["--not", &format!("--remotes={remote}")])
            .output()
            .ok()
            .filter(|ahead| ahead.status.success())?;
        let Ok(unpushed) = String::from_utf8_lossy(&ahead.stdout).lines().map(str::parse).collect();
        Some(unpushed)
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
