mod attempt;
mod content;
mod fetched;
mod listing;
mod outcome;
mod pacing;
mod remote;
mod remote_heads;
mod sha;
mod tip;

pub use remote_heads::RemoteHeads;
pub use sha::{NotCommitId, Sha};

use meta_git::{FileCommit, Files, Identity, Reads, RefMove, RefName, RefTarget, RemoteName, Repository, Writes};
use meta_run::{Budget, Stderr};
use meta_schema::Record;
use std::cell::RefCell;
use std::fmt;
use std::path::PathBuf;

use crate::LogStore;
use crate::log_failure::LogFailure;
use crate::snapshot::Snapshot;
use attempt::Attempt;
use content::Content;
use fetched::Fetched;
use listing::Listing;
use outcome::Outcome;
use pacing::Pacing;
use remote::Remote;
use tip::Tip;

#[derive(Clone, Debug)]
pub struct GitLog {
    repository: Repository,
    remote: Remote,
    known: RefCell<Option<Fetched>>,
    pacing: Pacing,
    budget: Budget,
}

impl GitLog {
    const STAGED: &'static str = "refs/meta-log/staged";
    const EMAIL_DOMAIN: &'static str = "@meta.invalid";

    pub fn new(repo: PathBuf, remote: RemoteName, budget: Budget) -> Result<Self, LogFailure> {
        let repository = Repository::open(&repo)?;
        let talk = meta_git::Remote::new(repo, remote, budget);
        Ok(Self {
            remote: Remote::new(repository.clone(), talk),
            repository,
            known: RefCell::default(),
            pacing: Pacing::Jittered,
            budget,
        })
    }

    pub fn fetched_tip(&self) -> Result<Option<Sha>, LogFailure> {
        self.remote.fetch_tip().map(Tip::sha)
    }

    pub fn snapshot_at(&self, tip: &Sha) -> Result<Snapshot, LogFailure> {
        Ok(self.remote.content_at(tip)?.snapshot())
    }

    pub fn read_listed(&self, heads: &RemoteHeads) -> Result<Snapshot, LogFailure> {
        Ok(self.remote.content_listed(heads)?.snapshot())
    }

    fn commit(&self, tip: Option<&Sha>, content: &Content, record: &Record) -> Result<Sha, LogFailure> {
        let actor = record.actor().to_string();
        let author = Identity::new(actor.clone(), format!("{actor}{}", Self::EMAIL_DOMAIN));
        let files: Files = [(Remote::FILE.to_owned(), Some(content.to_string()))].into_iter().collect();
        let parent = tip.map(Sha::object_id).transpose()?;
        let made = FileCommit::new(parent, files, author, format!("{actor}: {}", record.event()));
        let commit = self.repository.write(&made)?;
        let Ok(staged) = Self::STAGED.parse::<RefName>();
        let was = self.repository.read(&RefTarget::from(staged.clone()))?;
        self.repository.write(&RefMove::new(staged, was, commit))?;
        Ok(Sha::from(commit))
    }

    fn attempt<Failure: From<LogFailure>>(
        &self,
        decide: &mut dyn FnMut(&Snapshot) -> Result<Record, Failure>,
    ) -> Result<Outcome, Failure> {
        let known = self.known.borrow().clone();
        let before = known.map_or_else(|| self.fresh(Listing::Kept), Ok)?;
        let record = decide(&before.content().snapshot())?;
        let appended = before.content().appended(&record).map_err(LogFailure::from)?;
        let commit = self.commit(before.tip(), &appended, &record)?;
        match self.remote.push(&commit)? {
            Ok(()) => {
                self.known.replace(Some(Fetched::new(Some(commit), appended)));
                Ok(Outcome::Appended(record))
            }
            Err(stderr) => Ok(self.after_failed_push(&before, record, stderr)?),
        }
    }

    fn fresh(&self, listing: Listing) -> Result<Fetched, LogFailure> {
        let fetched = self.remote.fetch(listing)?;
        self.known.replace(Some(fetched.clone()));
        Ok(fetched)
    }

    fn after_failed_push(&self, before: &Fetched, record: Record, stderr: Stderr) -> Result<Outcome, LogFailure> {
        let now = self.fresh(Listing::Kept)?;
        match (now.content().snapshot().records().contains(&record), now.tip() == before.tip()) {
            (true, _) => Ok(Outcome::Appended(record)),
            (false, true) => Err(LogFailure::Rejected(stderr)),
            (false, false) => Ok(Outcome::Raced),
        }
    }
}

impl LogStore for GitLog {
    fn read(&self) -> Result<Snapshot, LogFailure> {
        Ok(self.fresh(Listing::Fresh)?.content().snapshot())
    }

    fn append<Failure: From<LogFailure>>(
        &self,
        decide: &mut dyn FnMut(&Snapshot) -> Result<Record, Failure>,
    ) -> Result<Record, Failure> {
        Attempt::sequence()
            .try_for_each(|attempt| Outcome::step(self.attempt(decide), attempt, self.pacing, &self.budget))
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
