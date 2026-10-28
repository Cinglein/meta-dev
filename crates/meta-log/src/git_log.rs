mod attempt;
mod content;
mod fetched;
mod git;
mod outcome;
mod pacing;
mod remote;
mod remote_heads;
mod sha;
mod tip;

pub use remote_heads::RemoteHeads;
pub use sha::{NotCommitId, Sha};

use std::cell::RefCell;
use std::fmt;
use std::path::PathBuf;
use std::process::Output;
use std::time::Instant;

use meta_schema::Record;

use crate::LogStore;
use crate::log_failure::LogFailure;
use crate::remote_name::RemoteName;
use crate::snapshot::Snapshot;
use crate::stderr::Stderr;
use attempt::Attempt;
use content::Content;
use fetched::Fetched;
use git::Git;
use outcome::Outcome;
use pacing::Pacing;
use remote::Remote;
use tip::Tip;

#[derive(Clone, Debug)]
pub struct GitLog {
    git: Git,
    remote: Remote,
    known: RefCell<Option<Fetched>>,
    pacing: Pacing,
}

impl GitLog {
    const STAGED: &'static str = "refs/meta-log/staged";

    #[must_use]
    pub fn new(repo: PathBuf, remote: RemoteName) -> Self {
        let git = Git::new(repo);
        Self { remote: Remote::new(git.clone(), remote), git, known: RefCell::default(), pacing: Pacing::Jittered }
    }

    pub fn fetched_tip(&self) -> Result<Option<Sha>, LogFailure> {
        self.remote.fetch_tip().map(Tip::sha)
    }

    pub fn snapshot_at(&self, tip: &Sha) -> Result<Snapshot, LogFailure> {
        Ok(self.remote.content_at(tip)?.snapshot())
    }

    pub fn read_listed(&self, heads: &RemoteHeads, deadline: Instant) -> Result<Snapshot, LogFailure> {
        Ok(self.remote.content_listed(heads, deadline)?.snapshot())
    }

    fn commit(&self, tip: Option<&Sha>, content: &Content, record: &Record) -> Result<Sha, LogFailure> {
        let actor = record.actor().to_string();
        let identity = format!("{actor} <{actor}@meta.invalid> now");
        let message = format!("{actor}: {}", record.event());
        let blob = content.to_string();
        let parent = tip.map(|parent| format!("from {parent}\n")).unwrap_or_default();
        let stream = format!(
            "blob\nmark :1\ndata {}\n{blob}\ncommit {staged}\nmark :2\nauthor {identity}\ncommitter {identity}\ndata {}\n{message}\n{parent}M 100644 :1 {}\n\nget-mark :2\nreset {staged}\n\n",
            blob.len(),
            message.len(),
            Remote::FILE,
            staged = Self::STAGED,
        );
        self.git.command(["fast-import", "--quiet", "--date-format=now"]).input(Content::from(stream)).sha()
    }

    fn attempt<Failure: From<LogFailure>>(
        &self,
        decide: &mut dyn FnMut(&Snapshot) -> Result<Record, Failure>,
    ) -> Result<Outcome, Failure> {
        let known = self.known.borrow().clone();
        let before = known.map_or_else(|| self.fresh(), Ok)?;
        let record = decide(&before.content().snapshot())?;
        let appended = before.content().appended(&record).map_err(LogFailure::from)?;
        let commit = self.commit(before.tip(), &appended, &record)?;
        let pushed = self.remote.push(&commit, before.tip())?;
        pushed
            .status
            .success()
            .then(|| {
                self.known.replace(Some(Fetched::new(Some(commit), appended)));
                Outcome::Appended(record.clone())
            })
            .map_or_else(|| self.after_failed_push(&before, record, &pushed), Ok)
            .map_err(Failure::from)
    }

    fn fresh(&self) -> Result<Fetched, LogFailure> {
        let fetched = self.remote.fetch()?;
        self.known.replace(Some(fetched.clone()));
        Ok(fetched)
    }

    fn after_failed_push(&self, before: &Fetched, record: Record, pushed: &Output) -> Result<Outcome, LogFailure> {
        let now = self.fresh()?;
        match (now.content().snapshot().records().contains(&record), now.tip() == before.tip()) {
            (true, _) => Ok(Outcome::Appended(record)),
            (false, true) => Err(LogFailure::Rejected(Stderr::from(pushed))),
            (false, false) => Ok(Outcome::Raced),
        }
    }
}

impl LogStore for GitLog {
    fn read(&self) -> Result<Snapshot, LogFailure> {
        Ok(self.fresh()?.content().snapshot())
    }

    fn append<Failure: From<LogFailure>>(
        &self,
        decide: &mut dyn FnMut(&Snapshot) -> Result<Record, Failure>,
    ) -> Result<Record, Failure> {
        Attempt::sequence()
            .try_for_each(|attempt| Outcome::step(self.attempt(decide), attempt, self.pacing))
            .break_value()
            .unwrap_or_else(|| Err(LogFailure::Contended.into()))
    }
}

impl fmt::Display for GitLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.remote)
    }
}

#[cfg(test)]
mod tests;
