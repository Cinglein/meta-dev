use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use meta_log::RemoteName;

use super::alternates::Alternates;
use super::inspection::Inspection;
use super::kept::Kept;
use super::vetted::Checkouts;

#[derive(Clone, Debug)]
pub struct Context {
    checkout: PathBuf,
    common: PathBuf,
    root: PathBuf,
    remote: RemoteName,
    checkouts: OnceLock<Result<Checkouts, Kept>>,
}

impl Context {
    const LINKED: &'static str = "worktrees";

    pub fn new(checkout: PathBuf, common: PathBuf, root: PathBuf, remote: RemoteName) -> Self {
        Self { checkout, common, root, remote, checkouts: OnceLock::new() }
    }

    pub fn checkouts(&self, host: &impl Inspection) -> &Result<Checkouts, Kept> {
        self.checkouts.get_or_init(|| Checkouts::listed(self, host))
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

    pub fn remote(&self) -> &RemoteName {
        &self.remote
    }
}
