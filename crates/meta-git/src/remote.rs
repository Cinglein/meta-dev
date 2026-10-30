use std::cell::RefCell;
use std::ffi::OsString;
use std::path::PathBuf;

use meta_run::{Budget, EnvPolicy, Ran, Run, Stderr};

use crate::asks::Asks;
use crate::git_failure::GitFailure;
use crate::queries::refs::Heads;
use crate::remote_change::RemoteChange;
use crate::remote_listing::RemoteListing;
use crate::remote_name::RemoteName;
use crate::remote_query::RemoteQuery;
use crate::sends::Sends;
use transport::Transport;

mod transport;

#[derive(Clone, Debug)]
pub struct Remote {
    directory: PathBuf,
    name: RemoteName,
    budget: Budget,
    heads: RefCell<Option<RemoteListing>>,
}

impl Remote {
    const PROGRAM: &'static str = "git";
    const DIRECTORY: &'static str = "-C";
    const NO_PROMPT: [&'static str; 2] = ["GIT_TERMINAL_PROMPT", "0"];

    #[must_use]
    pub fn new(directory: PathBuf, name: RemoteName, budget: Budget) -> Remote {
        Remote { directory, name, budget, heads: RefCell::default() }
    }

    #[must_use]
    pub fn name(&self) -> &RemoteName {
        &self.name
    }

    #[must_use]
    pub fn budget(&self) -> &Budget {
        &self.budget
    }

    pub fn heads(&self) -> Result<Result<RemoteListing, Stderr>, GitFailure> {
        let known = self.heads.borrow().clone();
        known.map_or_else(
            || {
                let listed =
                    self.local().map_or_else(|| self.ask(&Heads), |path| Ok(RemoteListing::heads_at(&path)))?;
                self.heads.replace(listed.as_ref().ok().cloned());
                Ok(listed)
            },
            |heads| Ok(Ok(heads)),
        )
    }

    pub fn forget_heads(&self) {
        self.heads.take();
    }

    fn local(&self) -> Option<PathBuf> {
        let here = git2::Repository::open(&self.directory).ok()?;
        let remote = here.find_remote(self.name.text()).ok()?;
        let Ok(transport) = remote.url().unwrap_or_default().parse::<Transport>();
        transport.local_path(&self.directory)
    }

    fn ran(&self, words: Vec<OsString>) -> Result<Ran, GitFailure> {
        let [prompt, off] = Self::NO_PROMPT;
        let run = Run::new(Self::PROGRAM, EnvPolicy::NoRepoVars).var(prompt, off);
        Ok(run.arg(Self::DIRECTORY).arg(&self.directory).args(words).ran(self.budget.deadline())?)
    }
}

impl Asks for Remote {
    fn ask<Asked: RemoteQuery>(&self, query: &Asked) -> Result<Asked::View, GitFailure> {
        query.view(self.ran(query.words(&self.name))?)
    }
}

impl Sends for Remote {
    fn send<Sent: RemoteChange>(&self, change: &Sent) -> Result<Result<(), Stderr>, GitFailure> {
        self.forget_heads();
        Ok(change.sent(self.ran(change.words(&self.name))?))
    }
}
