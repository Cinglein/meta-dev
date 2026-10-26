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
            let tracking = format!("refs/remotes/{}/{}", self.name, Self::BRANCH);
            let asked = Content::from(format!("info {tracking}^{{commit}}\ncontents {tracking}:{}\n", Self::FILE));
            let shown = self.git.command(["cat-file", "--batch-command"]).input(asked).content()?;
            Ok(Some(shown.to_string().parse().unwrap_or_else(|never| match never {})))
        } else {
            self.after_failed_fetch(&fetched)
        }
    }

    fn after_failed_fetch(&self, fetched: &Output) -> Result<Option<Fetched>, LogFailure> {
        let probe = self.git.command(["ls-remote"]).arg(self.name.to_string()).timed_output()?;
        probe.status.success().then_some(None).ok_or_else(|| LogFailure::Offline(Detail::from(fetched)))
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} on {}", Self::BRANCH, self.name)
    }
}
