use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use meta_log::RemoteName;

use super::alternates::Alternates;
use super::git_call::GitCall;
use super::look::Look;
use super::vetted::Checkouts;

// Where the clone lives, plus the lenders read once per sweep. A pure value: no reads happen here.
#[derive(Clone, Debug)]
pub struct Context {
    checkout: PathBuf,
    common: PathBuf,
    root: PathBuf,
    remote: RemoteName,
    lenders: Vec<Alternates>,
}

impl Context {
    const LINKED: &'static str = "worktrees";
    const REMOTE_REFS: &'static str = "ls-remote";

    pub fn new(checkout: PathBuf, common: PathBuf, root: PathBuf, remote: RemoteName) -> Self {
        Self { checkout, common, root, remote, lenders: Vec::new() }
    }

    pub fn lent(self, lenders: Vec<Alternates>) -> Self {
        Self { lenders, ..self }
    }

    pub fn worktrees(&self) -> GitCall {
        GitCall::common(&self.common, &Checkouts::LIST.map(OsStr::new))
    }

    pub fn remote_refs(&self) -> GitCall {
        GitCall::common(&self.common, &[OsStr::new(Self::REMOTE_REFS), self.remote.as_ref()])
    }

    pub fn lender_looks(&self) -> [Look; 2] {
        [Look::Lenders(self.root.clone()), Look::Lenders(self.common.join(Self::LINKED))]
    }

    pub fn enclosed(&self, path: &Path) -> Option<()> {
        (path.starts_with(&self.root)
            && path != self.root
            && !self.checkout.starts_with(path)
            && !self.common.starts_with(path))
        .then_some(())
    }

    pub fn lenders(&self) -> &[Alternates] {
        &self.lenders
    }

    pub fn common(&self) -> &Path {
        &self.common
    }
}
