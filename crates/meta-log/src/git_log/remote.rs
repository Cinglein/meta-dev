use std::fmt;
use std::process::Output;

use super::attempt::Attempt;
use super::content::Content;
use super::fetched::Fetched;
use super::git::Git;
use super::sha::Sha;
use crate::detail::Detail;
use crate::log_failure::LogFailure;

#[derive(Clone, Debug)]
pub struct Remote {
    git: Git,
    name: Detail,
}

impl Remote {
    pub const FILE: &'static str = "events.jsonl";
    const BRANCH: &'static str = "meta/log";

    pub fn new(git: Git, name: Detail) -> Self {
        Self { git, name }
    }

    pub fn fetch(&self) -> Result<Fetched, LogFailure> {
        Attempt::sequence()
            .find_map(|attempt| attempt.settled(self.fetch_once()))
            .unwrap_or_else(|| Err(LogFailure::Contended(Detail::from_reason(&Attempt::LIMIT))))
    }

    pub fn push(&self, commit: &Sha) -> Result<Output, LogFailure> {
        let target = format!("{commit}:refs/heads/{}", Self::BRANCH);
        self.git.command(["push", "--quiet"]).arg(self.name.to_string()).arg(target).timed_output()
    }

    fn fetch_once(&self) -> Result<Option<Fetched>, LogFailure> {
        let refspec = format!("+refs/heads/meta/*:refs/remotes/{}/meta/*", self.name);
        let fetched =
            self.git.command(["fetch", "--quiet", "--prune"]).arg(self.name.to_string()).arg(refspec).timed_output()?;
        if fetched.status.success() {
            let tip = self.tracking_tip();
            self.blob_at(tip.as_ref()).map(|content| Some(Fetched::new(tip, content)))
        } else {
            self.after_failed_fetch(&fetched)
        }
    }

    fn after_failed_fetch(&self, fetched: &Output) -> Result<Option<Fetched>, LogFailure> {
        let probe = self.git.command(["ls-remote"]).arg(self.name.to_string()).timed_output()?;
        probe.status.success().then_some(None).ok_or_else(|| LogFailure::Offline(Detail::from(fetched)))
    }

    fn tracking_tip(&self) -> Option<Sha> {
        let tracking = format!("refs/remotes/{}/{}^{{commit}}", self.name, Self::BRANCH);
        self.git.command(["rev-parse", "--verify", "--quiet"]).arg(tracking).sha().ok()
    }

    fn blob_at(&self, tip: Option<&Sha>) -> Result<Content, LogFailure> {
        Ok(tip
            .map(|commit| self.git.command(["cat-file", "blob"]).arg(format!("{commit}:{}", Self::FILE)).content())
            .transpose()?
            .unwrap_or_default())
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} on {}", Self::BRANCH, self.name)
    }
}
