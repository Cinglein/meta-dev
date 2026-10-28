use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use meta_log::RemoteName;

use super::alternates::Alternates;
use super::git_call::GitCall;
use super::inspection::Inspection;
use super::kept::Kept;
use super::vetted::Checkouts;

#[derive(Clone, Debug)]
pub struct Context {
    checkout: PathBuf,
    common: PathBuf,
    root: PathBuf,
    remote: RemoteName,
    worktrees: OnceLock<Result<String, Kept>>,
    checkouts: OnceLock<Result<Checkouts, Kept>>,
    remote_refs: OnceLock<Result<String, Kept>>,
}

impl Context {
    const LINKED: &'static str = "worktrees";
    const REMOTE_REFS: &'static str = "ls-remote";

    pub fn new(checkout: PathBuf, common: PathBuf, root: PathBuf, remote: RemoteName) -> Self {
        let [worktrees, remote_refs] = [OnceLock::new(), OnceLock::new()];
        Self { checkout, common, root, remote, worktrees, checkouts: OnceLock::new(), remote_refs }
    }

    pub fn worktrees(&self, host: &impl Inspection) -> &Result<String, Kept> {
        self.worktrees.get_or_init(|| host.git(&GitCall::common(&self.common, &Checkouts::LIST.map(OsStr::new))))
    }

    pub fn checkouts(&self, host: &impl Inspection) -> &Result<Checkouts, Kept> {
        self.checkouts.get_or_init(|| {
            let Ok(checkouts) = self.worktrees(host).as_ref().map_err(Kept::clone)?.parse();
            Ok(checkouts)
        })
    }

    pub fn remote_refs_now(&self, host: &impl Inspection) -> Result<String, Kept> {
        host.git(&GitCall::common(&self.common, &[OsStr::new(Self::REMOTE_REFS), self.remote.as_ref()]))
    }

    pub fn remote_refs(&self, host: &impl Inspection) -> &Result<String, Kept> {
        self.remote_refs.get_or_init(|| self.remote_refs_now(host))
    }

    pub fn enclosed(&self, path: &Path) -> Option<()> {
        (path.starts_with(&self.root)
            && path != self.root
            && !self.checkout.starts_with(path)
            && !self.common.starts_with(path))
        .then_some(())
    }

    pub fn lenders(&self, host: &impl Inspection) -> Vec<Alternates> {
        [host.neighbours(&self.root), host.neighbours(&self.common.join(Self::LINKED))].concat()
    }

    pub fn common(&self) -> &Path {
        &self.common
    }
}
