mod objects;

use std::fmt;

use meta_run::Ran;

use super::attempt::Attempt;
use super::content::Content;
use super::fetched::Fetched;
use super::git::Git;
use super::remote_heads::RemoteHeads;
use super::sha::Sha;
use super::tip::Tip;
use crate::log_failure::LogFailure;
use crate::remote_name::RemoteName;
use objects::Objects;

#[derive(Clone, Debug)]
pub struct Remote {
    git: Git,
    name: RemoteName,
}

impl Remote {
    pub const FILE: &'static str = "events.jsonl";
    pub const BRANCH: &'static str = "meta/log";

    pub fn new(git: Git, name: RemoteName) -> Self {
        Self { git, name }
    }

    pub fn fetch(&self) -> Result<Fetched, LogFailure> {
        let settled = |attempt: Attempt| attempt.settled(self.fetch_once(), &self.git);
        Attempt::sequence().find_map(settled).unwrap_or(Err(LogFailure::Contended))
    }

    pub fn fetch_tip(&self) -> Result<Tip, LogFailure> {
        self.fetch().map(|fetched| Tip::new(fetched.tip().cloned()))
    }

    pub fn content_at(&self, tip: &Sha) -> Result<Content, LogFailure> {
        self.held(Some(tip))?.ok_or_else(|| LogFailure::Unfetched(tip.clone()))
    }

    pub fn push(&self, commit: &Sha, base: Option<&Sha>) -> Result<Ran, LogFailure> {
        let target = format!("{commit}:refs/heads/{}", Self::BRANCH);
        let expected = base.map(ToString::to_string).unwrap_or_default();
        let lease = format!("--force-with-lease=refs/heads/{}:{expected}", Self::BRANCH);
        let pushed = self.git.command(["push", "--quiet"]).arg(lease).arg(self.name.to_string()).arg(target);
        self.git.ran(pushed)
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
        Ok(content.map(|content| Fetched::new(tip, content)))
    }

    fn fetched(&self) -> Result<Option<()>, LogFailure> {
        let refspec = format!("+refs/heads/meta/*:refs/remotes/{}/meta/*", self.name);
        let fetching =
            self.git.command(["fetch", "--quiet", "--prune", "--no-write-fetch-head", "--no-auto-maintenance"]);
        let fetched = self.git.ran(fetching.arg(self.name.to_string()).arg(refspec))?;
        Ok(fetched.success().then_some(()))
    }

    fn held(&self, tip: Option<&Sha>) -> Result<Option<Content>, LogFailure> {
        tip.map_or(Ok(Some(Content::default())), |tip| {
            let asked = self.git.command(["cat-file", "--batch-command"]).fed(Objects::request(tip).to_string());
            let answered = Content::try_from(self.git.ran(asked)?)?;
            answered.as_ref().parse::<Objects>()?.content(&self.name)
        })
    }

    fn listed_tip(&self) -> Result<Option<Sha>, LogFailure> {
        let branch = format!("refs/heads/{}", Self::BRANCH);
        let listing = self.git.command(["ls-remote"]).arg(self.name.to_string()).arg(&branch);
        let listed = self.git.ran(listing)?;
        let offline = || LogFailure::Offline { remote: self.name.clone(), stderr: listed.stderr().clone() };
        let lines = listed.success().then(|| listed.stdout().text());
        let Ok(heads) = lines.ok_or_else(offline)?.parse::<RemoteHeads>();
        Ok(heads.log_tip()?)
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} on {}", Self::BRANCH, self.name)
    }
}
