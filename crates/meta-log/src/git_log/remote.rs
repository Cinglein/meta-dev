mod objects;

use std::fmt;
use std::process::Output;

use super::attempt::Attempt;
use super::content::Content;
use super::fetched::Fetched;
use super::git::Git;
use super::sha::Sha;
use super::tip::Tip;
use crate::log_failure::LogFailure;
use crate::remote_name::RemoteName;
use crate::stderr::Stderr;
use objects::Objects;

#[derive(Clone, Debug)]
pub struct Remote {
    git: Git,
    name: RemoteName,
}

impl Remote {
    pub const FILE: &'static str = "events.jsonl";
    const BRANCH: &'static str = "meta/log";
    const SEPARATOR: char = '\t';

    pub fn new(git: Git, name: RemoteName) -> Self {
        Self { git, name }
    }

    pub fn fetch(&self) -> Result<Fetched, LogFailure> {
        Attempt::sequence().find_map(|attempt| attempt.settled(self.fetch_once())).unwrap_or(Err(LogFailure::Contended))
    }

    pub fn fetch_tip(&self) -> Result<Tip, LogFailure> {
        self.fetch().map(|fetched| Tip::new(fetched.tip().cloned()))
    }

    pub fn content_at(&self, tip: &Sha) -> Result<Content, LogFailure> {
        self.held(Some(tip))?.ok_or_else(|| LogFailure::Unfetched(tip.clone()))
    }

    pub fn push(&self, commit: &Sha) -> Result<Output, LogFailure> {
        let target = format!("{commit}:refs/heads/{}", Self::BRANCH);
        self.git.command(["push", "--quiet"]).arg(self.name.to_string()).arg(target).timed_output()
    }

    fn fetch_once(&self) -> Result<Option<Fetched>, LogFailure> {
        let tip = self.listed_tip()?;
        let held = self.held(tip.as_ref())?;
        let missing = || tip.clone().map_or(LogFailure::Contended, LogFailure::Unfetched);
        let refetched =
            || self.fetched()?.map_or(Ok(None), |()| self.held(tip.as_ref())?.ok_or_else(missing).map(Some));
        let content = held.map_or_else(refetched, |content| Ok(Some(content)))?;
        Ok(content.map(|content| Fetched::new(tip, content)))
    }

    fn fetched(&self) -> Result<Option<()>, LogFailure> {
        let refspec = format!("+refs/heads/meta/*:refs/remotes/{}/meta/*", self.name);
        let fetched = self
            .git
            .command(["fetch", "--quiet", "--prune", "--no-write-fetch-head", "--no-auto-maintenance"])
            .arg(self.name.to_string())
            .arg(refspec)
            .timed_output()?;
        Ok(fetched.status.success().then_some(()))
    }

    fn held(&self, tip: Option<&Sha>) -> Result<Option<Content>, LogFailure> {
        tip.map_or(Ok(Some(Content::default())), |tip| {
            let answered = self.git.command(["cat-file", "--batch-command"]).input(Objects::request(tip)).content()?;
            answered.as_ref().parse::<Objects>()?.content(&self.name)
        })
    }

    fn listed_tip(&self) -> Result<Option<Sha>, LogFailure> {
        let branch = format!("refs/heads/{}", Self::BRANCH);
        let listed = self.git.command(["ls-remote"]).arg(self.name.to_string()).arg(&branch).timed_output()?;
        let offline = || LogFailure::Offline { remote: self.name.clone(), stderr: Stderr::from(&listed) };
        let lines = listed.status.success().then(|| String::from_utf8_lossy(&listed.stdout).into_owned());
        let lines = lines.ok_or_else(offline)?;
        let tip = lines.lines().filter_map(|line| line.split_once(Self::SEPARATOR)).find(|(_, name)| *name == branch);
        Ok(tip.map(|(sha, _)| sha.parse().unwrap_or_else(|never| match never {})))
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} on {}", Self::BRANCH, self.name)
    }
}
