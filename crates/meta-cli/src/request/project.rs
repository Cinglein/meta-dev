mod config;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use meta_log::GitLog;

use crate::request::ci::{Commit, ReflogEntry, StartMoment};
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::Listed;
use crate::request::timed_command::TimedCommand;
pub use config::{BranchName, CiSettings, Config, GcMode, HeartbeatSettings, LogWindow, ProgramName, TickLimit};
#[cfg(test)]
mod tests;

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

    pub fn created_after(&self, start: &StartMoment) -> Option<Vec<Commit>> {
        let walked = ["log", "--walk-reflogs", "--date=unix", ReflogEntry::FORMAT, "HEAD", "--"].map(OsString::from);
        let entries: Vec<ReflogEntry> = self.git_lines(walked).filter(|entries| !entries.is_empty())?;
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

    pub fn remote_heads(&self) -> Result<Vec<Commit>, Listed> {
        let remote = self.config.remote().map_err(|failure| Listed::new(failure.to_string()))?;
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.root).args(["ls-remote", "--heads", &remote.to_string()]);
        let deadline = self.config.ci_settings().remote_deadline();
        let listed = TimedCommand::new(command, deadline).stdout().map_err(|failure| {
            Listed::new(format!("remote unreachable: git ls-remote {remote}: {failure}; retry, or ask the owner"))
        })?;
        Ok(listed.lines().filter_map(|line| line.parse().ok()).collect())
    }

    pub fn pushed_among(&self, commits: &[Commit], heads: &[Commit]) -> Vec<Commit> {
        let remote = commits.first().and_then(|_| self.config.remote().ok());
        let listed = commits.iter().map(Commit::argument);
        let ahead = remote.map(|remote| {
            let words = [OsString::from("rev-list"), OsString::from("--ignore-missing")].into_iter().chain(listed);
            let not = words.chain([OsString::from("--not"), OsString::from(format!("--remotes={remote}"))]);
            not.chain(heads.iter().map(Commit::argument))
        });
        let unpushed: Option<Vec<Commit>> = ahead.and_then(|words| self.git_lines(words));
        unpushed.map_or_else(Vec::new, |unpushed| {
            commits.iter().filter(|commit| !unpushed.contains(commit)).cloned().collect()
        })
    }

    pub fn independent_tips(&self, pushed: Vec<Commit>) -> Vec<Commit> {
        let independent = [OsString::from("merge-base"), OsString::from("--independent")];
        let tips = pushed.get(1).map(|_| independent.into_iter().chain(pushed.iter().map(Commit::argument)));
        tips.and_then(|words| self.git_lines(words)).unwrap_or(pushed)
    }

    pub fn head(&self) -> Option<Commit> {
        let heads: Vec<Commit> = self.git_lines(["rev-parse", "HEAD"].map(OsString::from))?;
        heads.into_iter().next()
    }

    fn git_lines<Line: FromStr>(&self, words: impl IntoIterator<Item = OsString>) -> Option<Vec<Line>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(words)
            .output()
            .ok()
            .filter(|output| output.status.success())?;
        Some(String::from_utf8_lossy(&output.stdout).lines().filter_map(|line| line.parse().ok()).collect())
    }

    fn born_after(&self, commit: &Commit, start: &StartMoment) -> bool {
        let words = [OsString::from("rev-parse"), OsString::from("--git-path"), commit.loose_object().into_os_string()];
        let paths: Option<Vec<PathBuf>> = self.git_lines(words);
        let path = paths.and_then(|paths| paths.into_iter().next()).map(|path| self.root.join(path));
        let modified = path.and_then(|path| std::fs::metadata(path).and_then(|meta| meta.modified()).ok());
        modified.is_some_and(|modified| start.precedes(modified))
    }

    pub fn unprotected_branch(&self) -> Option<BranchName> {
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
