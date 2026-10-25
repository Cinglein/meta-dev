mod attempt;
mod content;
mod git;
mod outcome;
mod sha;

use std::fmt;
use std::path::PathBuf;

use meta_schema::Record;

use crate::LogStore;
use crate::detail::Detail;
use crate::log_failure::LogFailure;
use crate::snapshot::Snapshot;
use attempt::Attempt;
use content::Content;
use git::Git;
use outcome::Outcome;
use sha::Sha;

#[derive(Clone, Debug)]
pub struct GitLog {
    git: Git,
    remote: Detail,
}

impl GitLog {
    const BRANCH: &'static str = "meta/log";
    const FILE: &'static str = "events.jsonl";

    pub fn new(repo: PathBuf, remote: impl ToString) -> Self {
        Self { git: Git::new(repo), remote: Detail::from_reason(remote) }
    }

    fn fetch(&self) -> Result<Option<Sha>, LogFailure> {
        let refspec = format!("+refs/heads/meta/*:refs/remotes/{}/meta/*", self.remote);
        let fetched =
            self.git.command(["fetch", "--quiet", "--prune"]).arg(self.remote.to_string()).arg(refspec).output()?;
        fetched.status.success().then_some(()).ok_or_else(|| LogFailure::Offline(Detail::from(&fetched)))?;
        let tracking = format!("refs/remotes/{}/{}^{{commit}}", self.remote, Self::BRANCH);
        let tip = self.git.command(["rev-parse", "--verify", "--quiet"]).arg(tracking).content().ok();
        Ok(tip.map(|content| content.to_string().parse().unwrap_or_else(|never| match never {})))
    }

    fn content(&self, tip: Option<&Sha>) -> Result<Content, LogFailure> {
        tip.map(|commit| self.git.command(["cat-file", "blob"]).arg(format!("{commit}:{}", Self::FILE)).content())
            .transpose()
            .map(|content| content.map(|text| Content::from(format!("{text}\n"))).unwrap_or_default())
    }

    fn commit(&self, tip: Option<&Sha>, content: Content, record: &Record) -> Result<Sha, LogFailure> {
        let blob = self.git.command(["hash-object", "-w", "--stdin"]).input(content).content()?;
        let entry = Content::from(format!("100644 blob {blob}\t{}\n", Self::FILE));
        let tree = self.git.command(["mktree"]).input(entry).content()?;
        let actor = record.actor().to_string();
        let email = format!("{actor}@meta.invalid");
        let parents = tip.iter().flat_map(|parent| ["-p".to_owned(), parent.to_string()]);
        let base = self
            .git
            .command(["commit-tree"])
            .arg(tree.to_string())
            .arg("-m")
            .arg(format!("{actor}: {}", record.event()));
        let commit = parents
            .fold(base, |command, word| command.arg(word))
            .env("GIT_AUTHOR_NAME", &actor)
            .env("GIT_AUTHOR_EMAIL", &email)
            .env("GIT_COMMITTER_NAME", &actor)
            .env("GIT_COMMITTER_EMAIL", &email)
            .content()?;
        Ok(commit.to_string().parse().unwrap_or_else(|never| match never {}))
    }

    fn attempt(&self, decide: &mut dyn FnMut(&Snapshot) -> Result<Record, LogFailure>) -> Result<Outcome, LogFailure> {
        let tip = self.fetch()?;
        let content = self.content(tip.as_ref())?;
        let record = decide(&content.snapshot())?;
        let commit = self.commit(tip.as_ref(), content.appended(&record)?, &record)?;
        let target = format!("{commit}:refs/heads/{}", Self::BRANCH);
        let pushed = self.git.command(["push", "--quiet"]).arg(self.remote.to_string()).arg(target).output()?;
        match (pushed.status.success(), self.fetch()? == tip) {
            (true, _) => Ok(Outcome::Appended(record)),
            (false, false) => Ok(Outcome::Raced),
            (false, true) => Err(LogFailure::Rejected(Detail::from(&pushed))),
        }
    }
}

impl LogStore for GitLog {
    fn read(&self) -> Result<Snapshot, LogFailure> {
        Ok(self.content(self.fetch()?.as_ref())?.snapshot())
    }

    fn append(&self, decide: &mut dyn FnMut(&Snapshot) -> Result<Record, LogFailure>) -> Result<Record, LogFailure> {
        Attempt::sequence()
            .try_for_each(|attempt| Outcome::step(self.attempt(decide), attempt))
            .break_value()
            .unwrap_or_else(|| Err(LogFailure::Contended(Detail::from_reason(Attempt::last()))))
    }
}

impl fmt::Display for GitLog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} on {}", Self::BRANCH, self.remote)
    }
}

#[cfg(test)]
mod tests;
