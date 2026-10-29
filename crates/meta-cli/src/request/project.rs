mod config;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Instant;

use meta_git::{Head, Reads as _, RemoteUrl};
use meta_log::{GitLog, RemoteHeads};
use meta_run::{Budget, GitArgv, Port, Run};

use crate::request::checkout::Checkout;
use crate::request::ci::{Commit, ReflogEntry, StartMoment};
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::output::Listed;
use crate::request::remote_failure::RemoteFailure;
use crate::request::repository::Repository;
pub use config::{BranchName, CiSettings, Config, GcMode, HeartbeatSettings, LogWindow, ProgramName, TickLimit};
#[cfg(test)]
mod tests;

const NO_OPERANDS: [&str; 0] = [];

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
        self.config.remote().and_then(|remote| Ok(GitLog::new(self.root.clone(), remote.clone(), budget)?))
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
        let unprotected = self.unprotected_branch()?;
        let push = self.git(["push", "--quiet", "-u", &remote, &unprotected.to_string()]);
        match push.ran(budget.remaining_deadline()) {
            Ok(ran) if ran.success() => None,
            Ok(ran) => Some(Listed::new(format!("push of {unprotected} failed: {}", ran.stderr()))),
            Err(failure) => Some(Listed::new(format!("push of {unprotected} failed: {failure}"))),
        }
    }

    pub fn reflog(&self, deadline: Instant) -> Option<&[ReflogEntry]> {
        self.reflog.get_or_init(|| self.lines(GitArgv::ReflogWalk, NO_OPERANDS, deadline)).as_deref()
    }

    pub fn created_after(&self, start: &StartMoment, deadline: Instant) -> Option<Vec<Commit>> {
        let entries = self.reflog(deadline).filter(|entries| !entries.is_empty())?;
        Some(ReflogEntry::created(entries, start, |commit| self.born_after(commit, start, deadline)))
    }

    pub fn remote_heads(&self, deadline: Instant) -> Result<RemoteHeads, Listed> {
        let remote = self.config.remote().map_err(|failure| Listed::new(failure.to_string()))?;
        let listed = Port::new(&self.root).reported(GitArgv::RemoteHeads, [remote.to_string()], deadline);
        let listed = listed.map_err(|failure| {
            Listed::new(format!("remote unreachable: git ls-remote {remote}: {failure}; retry, or ask the owner"))
        })?;
        let Ok(heads) = listed.text().parse();
        Ok(heads)
    }

    pub fn unpushed(&self, commits: &[Commit], heads: &[Commit], deadline: Instant) -> Option<Vec<Commit>> {
        let remote = commits.first().and_then(|_| self.config.remote().ok())?;
        let not = [OsString::from("--not"), OsString::from(format!("--remotes={remote}"))];
        let words = commits.iter().map(Commit::argument).chain(not).chain(heads.iter().map(Commit::argument));
        self.lines(GitArgv::Unpushed, words, deadline)
    }

    pub fn independent_tips(&self, pushed: &[Commit], deadline: Instant) -> Option<Vec<Commit>> {
        pushed.get(1)?;
        self.lines(GitArgv::IndependentTips, pushed.iter().map(Commit::argument), deadline)
    }

    pub fn head(&self) -> Option<Commit> {
        let heads: Vec<Commit> = self.lines(GitArgv::Head, NO_OPERANDS, self.ci.remote_deadline())?;
        heads.into_iter().next()
    }

    pub fn repository(&self) -> Result<Repository, RemoteFailure> {
        let remote = self.config.remote().map_err(|failure| RemoteFailure::NotGithub(Listed::new(failure)))?;
        let url = meta_git::Repository::open(&self.root)
            .and_then(|repository| repository.read(&RemoteUrl::from(remote.clone())));
        let url = url.map_err(|failure| {
            RemoteFailure::GitFailed(Listed::new(format!("git failed reading the remote URL: {failure}")))
        })?;
        let named = format!("remote URL `{url}` names no GitHub owner and repository");
        url.parse().map_err(|()| RemoteFailure::NotGithub(Listed::new(named)))
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

    fn lines<Line: FromStr>(
        &self,
        call: GitArgv,
        operands: impl IntoIterator<Item = impl AsRef<OsStr>>,
        deadline: Instant,
    ) -> Option<Vec<Line>> {
        let output = Port::new(&self.root).reported(call, operands, deadline).ok()?;
        Some(output.text().lines().filter_map(|line| line.parse().ok()).collect())
    }

    fn born_after(&self, commit: &Commit, start: &StartMoment, deadline: Instant) -> bool {
        let reported = Port::new(&self.root).reported(GitArgv::GitPath, [commit.loose_object()], deadline);
        let modified = |path: &Path| std::fs::metadata(self.root.join(path)).and_then(|meta| meta.modified()).ok();
        reported.is_ok_and(|git_path| start.born_after(&git_path.text(), modified))
    }

    pub fn checkout(&self) -> &Checkout {
        self.checkout.get_or_init(|| {
            let head = meta_git::Repository::open(&self.root).and_then(|repository| repository.read(&Head));
            head.map(|head| Checkout::from(&head)).unwrap_or_default()
        })
    }

    pub fn unprotected_branch(&self) -> Option<BranchName> {
        self.checkout().branch().filter(|branch| self.config.protects(branch).is_none()).cloned()
    }
}

impl TryFrom<&Path> for Project {
    type Error = CliFailure;

    fn try_from(dir: &Path) -> Result<Self, Self::Error> {
        Self::found(dir)?.ok_or_else(|| CliFailure::raised::<Setup>("not inside a git repository with a meta.toml"))
    }
}
