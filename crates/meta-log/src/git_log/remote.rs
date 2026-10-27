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

#[derive(Clone, Debug)]
pub struct Remote {
    git: Git,
    name: RemoteName,
}

impl Remote {
    pub const FILE: &'static str = "events.jsonl";
    const BRANCH: &'static str = "meta/log";

    pub fn new(git: Git, name: RemoteName) -> Self {
        Self { git, name }
    }

    pub fn fetch(&self) -> Result<Fetched, LogFailure> {
        let tip = self.fetch_tip()?.sha();
        self.blob_at(tip.as_ref()).map(|content| Fetched::new(tip, content))
    }

    pub fn fetch_tip(&self) -> Result<Tip, LogFailure> {
        Attempt::sequence().find_map(|attempt| attempt.settled(self.fetch_once())).unwrap_or(Err(LogFailure::Contended))
    }

    pub fn content_at(&self, tip: &Sha) -> Result<Content, LogFailure> {
        self.blob_at(Some(tip))
    }

    pub fn push(&self, commit: &Sha) -> Result<Output, LogFailure> {
        let target = format!("{commit}:refs/heads/{}", Self::BRANCH);
        self.git.command(["push", "--quiet"]).arg(self.name.to_string()).arg(target).timed_output()
    }

    fn fetch_once(&self) -> Result<Option<Tip>, LogFailure> {
        let refspec = format!("+refs/heads/meta/*:refs/remotes/{}/meta/*", self.name);
        let fetched = self
            .git
            .command(["fetch", "--quiet", "--prune", "--no-write-fetch-head", "--no-auto-maintenance"])
            .arg(self.name.to_string())
            .arg(refspec)
            .timed_output()?;
        if fetched.status.success() {
            Ok(Some(Tip::new(self.tracking_tip())))
        } else {
            self.after_failed_fetch(&fetched)
        }
    }

    fn after_failed_fetch(&self, fetched: &Output) -> Result<Option<Tip>, LogFailure> {
        let probe = self.git.command(["ls-remote"]).arg(self.name.to_string()).timed_output()?;
        probe
            .status
            .success()
            .then_some(None)
            .ok_or_else(|| LogFailure::Offline { remote: self.name.clone(), stderr: Stderr::from(fetched) })
    }

    fn tracking_tip(&self) -> Option<Sha> {
        let tracking = format!("refs/remotes/{}/{}^{{commit}}", self.name, Self::BRANCH);
        self.git.command(["rev-parse", "--verify", "--quiet"]).arg(tracking).sha().ok()
    }

    fn blob_at(&self, tip: Option<&Sha>) -> Result<Content, LogFailure> {
        tip.map_or(Ok(Content::default()), |commit| {
            let files = self.git.command(["ls-tree", "--name-only", "-z"]).arg(commit.to_string()).content()?;
            (files == Content::from(format!("{}\0", Self::FILE)))
                .then_some(())
                .ok_or_else(|| LogFailure::Malformed(self.name.clone()))?;
            self.git.command(["cat-file", "blob"]).arg(format!("{commit}:{}", Self::FILE)).content()
        })
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} on {}", Self::BRANCH, self.name)
    }
}
