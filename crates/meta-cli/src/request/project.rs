mod config;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Instant;

use meta_door::{Door, MetaStore};
use meta_log::{GitLog, RemoteHeads};

use crate::request::checkout::Checkout;
use crate::request::ci::{Commit, ReflogEntry, StartMoment};
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::Listed;
use crate::request::remote_failure::RemoteFailure;
use crate::request::repository::Repository;
use crate::request::timed_command::TimedCommand;
pub use config::{BranchName, CiSettings, Config, GcMode, HeartbeatSettings, LogWindow, ProgramName, TickLimit};
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    config: Config,
    checkout: OnceLock<Checkout>,
}

impl Project {
    const CONFIG_FILE: &'static str = "meta.toml";
    pub fn log(&self) -> Result<GitLog, CliFailure> {
        self.config.remote().map(|remote| GitLog::new(self.root.clone(), remote.clone()))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn push_failure(&self) -> Option<Listed> {
        let remote = self.config.remote().ok()?.to_string();
        let unprotected = self.unprotected_branch()?;
        let pushed = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["push", "--quiet", "-u", &remote, &unprotected.to_string()])
            .output()
            .ok()?;
        (!pushed.status.success()).then(|| {
            Listed::new(format!("push of {unprotected} failed: {}", String::from_utf8_lossy(&pushed.stderr).trim()))
        })
    }

    pub fn created_after(&self, start: &StartMoment, deadline: Instant) -> Option<Vec<Commit>> {
        let walked = ["log", "--walk-reflogs", "--date=unix", ReflogEntry::FORMAT, "HEAD", "--"].map(OsString::from);
        let entries: Vec<ReflogEntry> = self.git_lines(walked, deadline).filter(|entries| !entries.is_empty())?;
        let befores = entries.iter().skip(1).map(Some).chain([None]);
        let created: Vec<Commit> = entries
            .iter()
            .zip(befores)
            .filter_map(|(entry, before)| {
                entry.created_after(start, before, |commit| self.born_after(commit, start)).cloned()
            })
            .collect();
        Some(
            created
                .iter()
                .enumerate()
                .filter(|(at, commit)| created.iter().position(|other| other == *commit) == Some(*at))
                .map(|(_, commit)| commit.clone())
                .collect(),
        )
    }

    pub fn remote_heads(&self, deadline: Instant) -> Result<RemoteHeads, Listed> {
        let remote = self.config.remote().map_err(|failure| Listed::new(failure.to_string()))?;
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.root).args(["ls-remote", "--heads", &remote.to_string()]);
        let listed = TimedCommand::new(command, deadline).stdout().map_err(|failure| {
            Listed::new(format!("remote unreachable: git ls-remote {remote}: {failure}; retry, or ask the owner"))
        })?;
        let Ok(heads) = listed.parse();
        Ok(heads)
    }

    pub fn pushed_among(&self, commits: &[Commit], heads: &[Commit], deadline: Instant) -> Vec<Commit> {
        let remote = commits.first().and_then(|_| self.config.remote().ok());
        let listed = commits.iter().map(Commit::argument);
        let ahead = remote.map(|remote| {
            let words = [OsString::from("rev-list"), OsString::from("--ignore-missing")].into_iter().chain(listed);
            let not = words.chain([OsString::from("--not"), OsString::from(format!("--remotes={remote}"))]);
            not.chain(heads.iter().map(Commit::argument))
        });
        let unpushed: Option<Vec<Commit>> = ahead.and_then(|words| self.git_lines(words, deadline));
        unpushed.map_or_else(Vec::new, |unpushed| {
            commits.iter().filter(|commit| !unpushed.contains(commit)).cloned().collect()
        })
    }

    pub fn independent_tips(&self, pushed: Vec<Commit>, deadline: Instant) -> Vec<Commit> {
        let independent = [OsString::from("merge-base"), OsString::from("--independent")];
        let tips = pushed.get(1).map(|_| independent.into_iter().chain(pushed.iter().map(Commit::argument)));
        tips.and_then(|words| self.git_lines(words, deadline)).unwrap_or(pushed)
    }

    pub fn head(&self) -> Option<Commit> {
        let deadline = self.config.ci_settings().remote_deadline();
        let heads: Vec<Commit> = self.git_lines(["rev-parse", "HEAD"].map(OsString::from), deadline)?;
        heads.into_iter().next()
    }

    pub fn repository(&self) -> Result<Repository, RemoteFailure> {
        let deadline = self.config.ci_settings().remote_deadline();
        let remote = self.config.remote().map_err(|failure| RemoteFailure::NotGithub(Listed::new(failure)))?;
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.root).args(["remote", "get-url"]).arg(remote.to_string());
        let url = TimedCommand::new(command, deadline).stdout().map_err(|failure| {
            RemoteFailure::GitFailed(Listed::new(format!("git failed reading the remote URL: {failure}")))
        })?;
        let named = format!("remote URL `{}` names no GitHub owner and repository", url.trim());
        url.lines().next().unwrap_or_default().parse().map_err(|()| RemoteFailure::NotGithub(Listed::new(named)))
    }

    fn git_lines<Line: FromStr>(
        &self,
        words: impl IntoIterator<Item = OsString>,
        deadline: Instant,
    ) -> Option<Vec<Line>> {
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.root).args(words);
        let output = TimedCommand::new(command, deadline).stdout().ok()?;
        Some(output.lines().filter_map(|line| line.parse().ok()).collect())
    }

    fn born_after(&self, commit: &Commit, start: &StartMoment) -> bool {
        let admin = MetaStore::located(&self.root).map(|store| store.admin());
        let object = admin.and_then(|admin| admin.described(&admin.path().join(commit.loose_object())));
        object.and_then(|object| object.modified()).is_ok_and(|modified| start.precedes(modified))
    }

    pub fn checkout(&self, deadline: Instant) -> &Checkout {
        self.checkout.get_or_init(|| Checkout::read(&self.root, deadline))
    }

    pub fn unprotected_branch(&self) -> Option<BranchName> {
        let checkout = self.checkout(self.config.ci_settings().remote_deadline());
        let unborn = || {
            let current =
                Command::new("git").arg("-C").arg(&self.root).args(["branch", "--show-current"]).output().ok()?;
            String::from_utf8_lossy(&current.stdout).trim().parse().ok()
        };
        let branch: BranchName = checkout.head().map_or_else(unborn, |_| checkout.branch().cloned())?;
        Some(branch).filter(|branch| self.config.protects(branch).is_none())
    }
}

impl TryFrom<&Path> for Project {
    type Error = CliFailure;

    fn try_from(dir: &Path) -> Result<Self, Self::Error> {
        let root = Door::repository(dir).ok_or_else(|| CliFailure::raised::<Setup>("not inside a git repository"))?;
        let path = root.join(Self::CONFIG_FILE);
        let text = Door::read(&path).map_err(|e| CliFailure::raised::<Setup>(format!("{}: {e}", path.display())))?;
        let config =
            toml::from_str(&text).map_err(|e| CliFailure::raised::<Setup>(format!("{}: {e}", path.display())))?;
        Ok(Self { root, config, checkout: OnceLock::new() })
    }
}
