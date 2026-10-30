use std::cell::RefCell;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use meta_run::{Budget, EnvPolicy, Ran, Run, Stderr};

use crate::asks::Asks;
use crate::git_failure::GitFailure;
use crate::queries::refs::Heads;
use crate::remote_change::RemoteChange;
use crate::remote_listing::RemoteListing;
use crate::remote_name::RemoteName;
use crate::remote_query::RemoteQuery;
use crate::sends::Sends;
pub use transport::Transport;

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

    const PACKS: &'static str = "objects/pack";
    const SECTION: &'static str = "remote.";
    const DOT: &'static str = ".";
    const NO_PACKS: &'static str = "cannot create objects/pack to receive objects";

    #[must_use]
    pub fn new(directory: PathBuf, name: RemoteName, budget: Budget) -> Remote {
        Remote { directory, name, budget, heads: RefCell::default() }
    }

    fn configured_helper(&self, side: &str) -> Option<String> {
        let key = [Self::SECTION, self.name.text(), Self::DOT, side].concat();
        git2::Repository::open(&self.directory).ok()?.config().ok()?.get_string(&key).ok()
    }

    fn packable(path: &Path) -> Result<git2::Repository, git2::Error> {
        let packs = git2::Repository::open(path)?.commondir().join(Self::PACKS);
        std::fs::create_dir_all(packs).map_err(|_| git2::Error::from_str(Self::NO_PACKS))?;
        git2::Repository::open(path)
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
                let there = self.transport().opened();
                let listed = there.map_or_else(|| self.ask(&Heads), |there| Ok(RemoteListing::heads_in(&there)))?;
                self.heads.replace(listed.as_ref().ok().cloned());
                Ok(listed)
            },
            |heads| Ok(Ok(heads)),
        )
    }

    pub fn forget_heads(&self) {
        self.heads.take();
    }

    #[must_use]
    pub fn local_end<Sent: RemoteChange>(&self, change: &Sent) -> Option<git2::Repository> {
        let local = self.transport().opened().filter(|_| self.configured_helper(Sent::HELPER).is_none());
        local.and_then(|there| change.in_process(there))
    }

    #[must_use]
    pub fn transport(&self) -> Transport {
        let here = git2::Repository::open(&self.directory).ok();
        let url = here.and_then(|here| here.find_remote(self.name.text()).ok()?.url().ok().map(str::to_owned));
        let Ok(transport) = url.unwrap_or_default().parse::<Transport>();
        transport.below(&self.directory)
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
        match self.local_end(change) {
            Some(there) => {
                let [here, there] = [self.directory.as_path(), there.path()].map(Self::packable);
                let moved = here.and_then(|here| change.moved(&here, &there?));
                Ok(moved.map_err(|error| Stderr::said(error.message().to_owned())))
            }
            None => Ok(change.sent(self.ran(change.words(&self.name))?)),
        }
    }
}
