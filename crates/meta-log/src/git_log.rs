mod attempt;
mod content;
mod fetched;
mod git;
mod outcome;
mod remote;
mod sha;

use std::fmt;
use std::path::PathBuf;
use std::process::Output;

use meta_schema::Record;

use crate::LogStore;
use crate::detail::Detail;
use crate::log_failure::LogFailure;
use crate::snapshot::Snapshot;
use attempt::Attempt;
use content::Content;
use fetched::Fetched;
use git::Git;
use outcome::Outcome;
use remote::Remote;
use sha::Sha;

#[derive(Clone, Debug)]
pub struct GitLog {
    git: Git,
    remote: Remote,
}

impl GitLog {
    const STAGED: &'static str = "refs/meta-log/staged";

    pub fn new(repo: PathBuf, remote: &impl ToString) -> Self {
        let git = Git::new(repo);
        Self { remote: Remote::new(git.clone(), Detail::from_reason(remote)), git }
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
        let commit =
            self.git.command(["fast-import", "--quiet", "--date-format=now"]).input(Content::from(stream)).content()?;
        Ok(commit.to_string().parse().unwrap_or_else(|never| match never {}))
    }

    fn attempt(&self, decide: &mut dyn FnMut(&Snapshot) -> Result<Record, LogFailure>) -> Result<Outcome, LogFailure> {
        let before = self.remote.fetch()?;
        let record = decide(&before.content().snapshot())?;
        let commit = self.commit(before.tip(), &before.content().appended(&record)?, &record)?;
        let pushed = self.remote.push(&commit)?;
        pushed
            .status
            .success()
            .then(|| Outcome::Appended(record.clone()))
            .map_or_else(|| self.after_failed_push(&before, record, &pushed), Ok)
    }

    fn after_failed_push(&self, before: &Fetched, record: Record, pushed: &Output) -> Result<Outcome, LogFailure> {
        let now = self.remote.fetch()?;
        match (now.content().snapshot().records().contains(&record), now.tip() == before.tip()) {
            (true, _) => Ok(Outcome::Appended(record)),
            (false, true) => Err(LogFailure::Rejected(Detail::from(pushed))),
            (false, false) => Ok(Outcome::Raced),
        }
    }
}

impl LogStore for GitLog {
    fn read(&self) -> Result<Snapshot, LogFailure> {
        Ok(self.remote.fetch()?.content().snapshot())
    }

    fn append(&self, decide: &mut dyn FnMut(&Snapshot) -> Result<Record, LogFailure>) -> Result<Record, LogFailure> {
        Attempt::sequence()
            .try_for_each(|attempt| Outcome::step(self.attempt(decide), attempt))
            .break_value()
            .unwrap_or_else(|| Err(LogFailure::Contended(Detail::from_reason(&Attempt::last()))))
    }
}

impl fmt::Display for GitLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.remote)
    }
}

#[cfg(test)]
mod tests;
