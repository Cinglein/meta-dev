mod config;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Instant;

use meta_log::{GitLog, RemoteHeads};
use meta_run::{Budget, Run};

use crate::request::checkout::Checkout;
use crate::request::ci::{Commit, ReflogEntry, StartMoment};
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::Listed;
use crate::request::remote_failure::RemoteFailure;
use crate::request::repository::Repository;
pub use config::{BranchName, CiSettings, Config, GcMode, HeartbeatSettings, LogWindow, ProgramName, TickLimit};
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    config: Config,
    ci: CiSettings,
    checkout: OnceLock<Checkout>,
    reflog: OnceLock<Option<Vec<ReflogEntry>>>,
}

impl Project {
    const CONFIG_FILE: &'static str = "meta.toml";
    const GIT_ENTRY: &'static str = ".git";
    pub fn log(&self, budget: Budget) -> Result<GitLog, CliFailure> {
        self.config.remote().map(|remote| GitLog::new(self.root.clone(), remote.clone(), budget))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn found(dir: &Path) -> Result<Option<Self>, CliFailure> {
        let root = dir.ancestors().find(|ancestor| ancestor.join(Self::GIT_ENTRY).exists()).map(Path::to_path_buf);
        let configured = root.filter(|root| root.join(Self::CONFIG_FILE).exists());
        configured.map(Self::loaded).transpose()
    }

    pub fn ci_settings(&self) -> &CiSettings {
        &self.ci
    }

    pub fn push_failure(&self, budget: Budget) -> Option<Listed> {
        let remote = self.config.remote().ok()?.to_string();
        let unprotected = self.unprotected_branch(budget.deadline())?;
        let push = self.git(["push", "--quiet", "-u", &remote, &unprotected.to_string()]);
        match push.ran(budget.deadline()) {
            Ok(ran) if ran.success() => None,
            Ok(ran) => Some(Listed::new(format!("push of {unprotected} failed: {}", ran.stderr()))),
            Err(failure) => Some(Listed::new(format!("push of {unprotected} failed: {failure}"))),
        }
    }

    pub fn reflog(&self, deadline: Instant) -> Option<&[ReflogEntry]> {
        self.reflog.get_or_init(|| self.git_lines(ReflogEntry::WALK.map(OsString::from), deadline)).as_deref()
    }

    pub fn created_after(&self, start: &StartMoment, deadline: Instant) -> Option<Vec<Commit>> {
        let entries = self.reflog(deadline).filter(|entries| !entries.is_empty())?;
        let befores = entries.iter().skip(1).map(Some).chain([None]);
        let created: Vec<Commit> = entries
            .iter()
            .zip(befores)
            .filter_map(|(entry, before)| {
                entry.created_after(start, before, |commit| self.born_after(commit, start, deadline)).cloned()
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
        let listed = self.git(["ls-remote", "--heads", &remote.to_string()]);
        let listed = listed.stdout(deadline).map_err(|failure| {
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
        let deadline = self.ci.remote_deadline();
        let heads: Vec<Commit> = self.git_lines(["rev-parse", "HEAD"].map(OsString::from), deadline)?;
        heads.into_iter().next()
    }

    pub fn repository(&self) -> Result<Repository, RemoteFailure> {
        let deadline = self.ci.remote_deadline();
        let remote = self.config.remote().map_err(|failure| RemoteFailure::NotGithub(Listed::new(failure)))?;
        let url = self.git(["remote", "get-url"]).arg(remote.to_string()).stdout(deadline).map_err(|failure| {
            RemoteFailure::GitFailed(Listed::new(format!("git failed reading the remote URL: {failure}")))
        })?;
        let named = format!("remote URL `{}` names no GitHub owner and repository", url.trim());
        url.lines().next().unwrap_or_default().parse().map_err(|()| RemoteFailure::NotGithub(Listed::new(named)))
    }

    fn loaded(root: PathBuf) -> Result<Self, CliFailure> {
        let path = root.join(Self::CONFIG_FILE);
        let refused =
            |failure: &dyn std::fmt::Display| CliFailure::raised::<Setup>(format!("{}: {failure}", path.display()));
        let text = std::fs::read_to_string(&path).map_err(|failure| refused(&failure))?;
        let config = toml::from_str(&text).map_err(|failure| refused(&failure))?;
        let ci = CiSettings::try_from(&config)?;
        Ok(Self { root, config, ci, checkout: OnceLock::new(), reflog: OnceLock::new() })
    }

    fn git<Word: AsRef<OsStr>>(&self, words: impl IntoIterator<Item = Word>) -> Run {
        Run::git(&self.root).args(words)
    }

    fn git_lines<Line: FromStr>(
        &self,
        words: impl IntoIterator<Item = OsString>,
        deadline: Instant,
    ) -> Option<Vec<Line>> {
        let output = self.git(words).stdout(deadline).ok()?;
        Some(output.lines().filter_map(|line| line.parse().ok()).collect())
    }

    fn born_after(&self, commit: &Commit, start: &StartMoment, deadline: Instant) -> bool {
        let words = [OsString::from("rev-parse"), OsString::from("--git-path"), commit.loose_object().into_os_string()];
        let paths: Option<Vec<PathBuf>> = self.git_lines(words, deadline);
        let path = paths.and_then(|paths| paths.into_iter().next()).map(|path| self.root.join(path));
        let modified = path.and_then(|path| std::fs::metadata(path).and_then(|meta| meta.modified()).ok());
        modified.is_some_and(|modified| start.precedes(modified))
    }

    pub fn checkout(&self, deadline: Instant) -> &Checkout {
        self.checkout.get_or_init(|| {
            let decorated = self.reflog(deadline).and_then(|entries| entries.iter().find_map(ReflogEntry::checkout));
            decorated.cloned().unwrap_or_default()
        })
    }

    pub fn unprotected_branch(&self, deadline: Instant) -> Option<BranchName> {
        let checkout = self.checkout(deadline);
        let unborn = || self.git(["branch", "--show-current"]).stdout(deadline).ok()?.trim().parse().ok();
        let branch: BranchName = checkout.head().map_or_else(unborn, |_| checkout.branch().cloned())?;
        Some(branch).filter(|branch| self.config.protects(branch).is_none())
    }
}

impl TryFrom<&Path> for Project {
    type Error = CliFailure;

    fn try_from(dir: &Path) -> Result<Self, Self::Error> {
        Self::found(dir)?.ok_or_else(|| CliFailure::raised::<Setup>("not inside a git repository with a meta.toml"))
    }
}
