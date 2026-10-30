mod config;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Instant;

use meta_git::{Head, HeadReflog, IndependentTips, LooseObject, Query, Reads as _, Unpushed};
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
        self.config.remote().and_then(|remote| Ok(GitLog::new(self.root.clone(), remote.clone(), budget)?))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn found(dir: &Path) -> Result<Option<Self>, CliFailure> {
        Self::located(dir).map(Self::loaded).transpose()
    }

    pub fn located(dir: &Path) -> Option<PathBuf> {
        let root = dir.ancestors().find(|ancestor| ancestor.join(Self::GIT_ENTRY).exists()).map(Path::to_path_buf);
        root.filter(|root| root.join(Self::CONFIG_FILE).exists())
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

    pub fn reflog(&self) -> Option<&[ReflogEntry]> {
        self.reflog.get_or_init(|| self.read(&HeadReflog).and_then(|lines| ReflogEntry::read(&lines).ok())).as_deref()
    }

    pub fn created_after(&self, start: &StartMoment) -> Option<Vec<Commit>> {
        let entries = self.reflog().filter(|entries| !entries.is_empty())?;
        Some(ReflogEntry::created(entries, start, |commit| self.born_after(commit, start)))
    }

    pub fn remote_heads(&self, deadline: Instant) -> Result<RemoteHeads, Listed> {
        let remote = self.config.remote().map_err(|failure| Listed::new(failure.to_string()))?;
        let budget = Budget::new(deadline.saturating_duration_since(Instant::now())).within(deadline);
        let listed = meta_git::Remote::new(self.root.clone(), remote.clone(), budget).heads();
        let unreachable = |failure: &dyn std::fmt::Display| {
            Listed::new(format!(
                "remote unreachable: listing the heads of {remote}: {failure}; retry, or ask the owner"
            ))
        };
        let listed = listed.map_err(|failure| unreachable(&failure))?.map_err(|stderr| unreachable(&stderr))?;
        let Ok(heads) = listed.to_string().parse();
        Ok(heads)
    }

    pub fn unpushed(&self, commits: &[Commit], heads: &[Commit]) -> Option<Vec<Commit>> {
        let remote = commits.first().and_then(|_| self.config.remote().ok())?;
        let ids = |commits: &[Commit]| commits.iter().filter_map(Commit::object_id).collect();
        let unpushed = self.read(&Unpushed::new(ids(commits), remote.clone(), ids(heads)))?;
        Some(unpushed.into_iter().map(Commit::from).collect())
    }

    pub fn independent_tips(&self, pushed: &[Commit]) -> Option<Vec<Commit>> {
        pushed.get(1)?;
        let tips =
            self.read(&IndependentTips::from(pushed.iter().filter_map(Commit::object_id).collect::<Vec<_>>()))?;
        Some(tips.into_iter().map(Commit::from).collect())
    }

    pub fn head(&self) -> Option<Commit> {
        self.checkout().head().cloned()
    }

    pub fn repository(&self) -> Result<Repository, RemoteFailure> {
        let remote = self.config.remote().map_err(|failure| RemoteFailure::NotGithub(Listed::new(failure)))?;
        let read = meta_git::Repository::open(&self.root)
            .and_then(|repository| repository.read(&meta_git::RemoteUrl::from(remote.clone())));
        let url = read.map(Option::unwrap_or_default).map_err(|failure| {
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

    fn read<Asked: Query>(&self, query: &Asked) -> Option<Asked::View> {
        meta_git::Repository::open(&self.root).and_then(|repository| repository.read(query)).ok()
    }

    fn born_after(&self, commit: &Commit, start: &StartMoment) -> bool {
        let path = commit.object_id().and_then(|object| self.read(&LooseObject::from(object)));
        let modified = |path: &Path| std::fs::metadata(path).and_then(|meta| meta.modified()).ok();
        path.is_some_and(|path| start.born_after(&path, modified))
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
