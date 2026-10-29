use std::fmt;

use meta_git::{FastForward, Fetch, FilesAt, Reads, Repository, Sends};
use meta_run::Stderr;

use super::attempt::Attempt;
use super::content::Content;
use super::fetched::Fetched;
use super::listing::Listing;
use super::remote_heads::RemoteHeads;
use super::sha::Sha;
use super::tip::Tip;
use crate::log_failure::LogFailure;

#[derive(Clone, Debug)]
pub struct Remote {
    repository: Repository,
    talk: meta_git::Remote,
}

impl Remote {
    pub const FILE: &'static str = "events.jsonl";
    pub const BRANCH: &'static str = "meta/log";
    const BRANCH_REF: &'static str = "refs/heads/meta/log";

    pub fn new(repository: Repository, talk: meta_git::Remote) -> Self {
        Self { repository, talk }
    }

    pub fn fetch(&self, listing: Listing) -> Result<Fetched, LogFailure> {
        (listing == Listing::Fresh).then(|| self.talk.forget_heads());
        let budget = self.talk.budget();
        Attempt::within(budget.deadline(), budget, || self.fetch_once()).unwrap_or(Err(LogFailure::Contended))
    }

    pub fn fetch_tip(&self) -> Result<Tip, LogFailure> {
        self.fetch(Listing::Fresh).map(|fetched| Tip::new(fetched.tip().cloned()))
    }

    pub fn content_at(&self, tip: &Sha) -> Result<Content, LogFailure> {
        self.held(Some(tip))?.ok_or_else(|| LogFailure::Unfetched(tip.clone()))
    }

    pub fn push(&self, commit: &Sha) -> Result<Result<(), Stderr>, LogFailure> {
        let Ok(branch) = Self::BRANCH_REF.parse();
        Ok(self.talk.send(&FastForward::new(commit.object_id()?, branch))?)
    }

    pub fn content_listed(&self, heads: &RemoteHeads) -> Result<Content, LogFailure> {
        let tip = heads.log_tip()?;
        let missing = || tip.clone().map_or(LogFailure::Contended, LogFailure::Unfetched);
        let refetched = || self.fetched()?.ok_or_else(missing).and_then(|()| self.held(tip.as_ref()));
        self.held(tip.as_ref())?.map_or_else(refetched, |content| Ok(Some(content)))?.ok_or_else(missing)
    }

    fn fetch_once(&self) -> Result<Option<Fetched>, LogFailure> {
        let tip = self.listed_tip()?;
        let held = self.held(tip.as_ref())?;
        let missing = || tip.clone().map_or(LogFailure::Contended, LogFailure::Unfetched);
        let refetched = || {
            let fetched = self.fetched()?;
            fetched.map_or(Ok(None), |()| self.held(tip.as_ref())?.ok_or_else(missing).map(Some))
        };
        let content = held.map_or_else(refetched, |content| Ok(Some(content)))?;
        content.is_none().then(|| self.talk.forget_heads());
        Ok(content.map(|content| Fetched::new(tip, content)))
    }

    fn fetched(&self) -> Result<Option<()>, LogFailure> {
        let refspec = format!("+refs/heads/meta/*:refs/remotes/{}/meta/*", self.talk.name());
        Ok(self.talk.send(&Fetch::from(refspec))?.ok())
    }

    fn held(&self, tip: Option<&Sha>) -> Result<Option<Content>, LogFailure> {
        tip.map_or(Ok(Some(Content::default())), |tip| {
            let files = self.repository.read(&FilesAt::from(tip.object_id()?))?;
            let malformed = || LogFailure::Malformed(self.talk.name().clone());
            let content = |text: &str| Content::from(text.to_owned());
            files.map(|files| files.only(Self::FILE).map(content).ok_or_else(malformed)).transpose()
        })
    }

    fn listed_tip(&self) -> Result<Option<Sha>, LogFailure> {
        let Ok(branch) = Self::BRANCH_REF.parse();
        let offline = |stderr| LogFailure::Offline { remote: self.talk.name().clone(), stderr };
        let heads = self.talk.heads()?.map_err(offline)?;
        Ok(heads.target(&branch).map(Sha::from))
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} on {}", Self::BRANCH, self.talk.name())
    }
}
