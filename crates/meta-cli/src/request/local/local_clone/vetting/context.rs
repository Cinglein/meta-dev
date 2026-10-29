use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use meta_log::RemoteName;

use super::alternates::Alternates;
use super::git_call::GitCall;
use super::inspection::Inspection;
use super::kept::Kept;
use super::vetted::{Checkouts, RefList};
use crate::request::local::local_clone::worktree_listing::WorktreeListing;

#[derive(Clone, Debug)]
pub struct Context {
    checkout: PathBuf,
    common: PathBuf,
    roots: Vec<PathBuf>,
    remote: RemoteName,
    worktrees: OnceLock<Result<WorktreeListing, Kept>>,
    checkouts: OnceLock<Result<Checkouts, Kept>>,
    remote_refs: OnceLock<Result<RefList, Kept>>,
}

impl Context {
    const LINKED: &'static str = "worktrees";
    const REMOTE_REFS: &'static str = "ls-remote";

    pub fn new(checkout: PathBuf, common: PathBuf, roots: Vec<PathBuf>, remote: RemoteName) -> Self {
        Self {
            checkout,
            common,
            roots,
            remote,
            worktrees: OnceLock::new(),
            checkouts: OnceLock::new(),
            remote_refs: OnceLock::new(),
        }
    }

    pub fn worktrees(&self, host: &impl Inspection) -> &Result<WorktreeListing, Kept> {
        self.worktrees.get_or_init(|| {
            host.git(&GitCall::common(&self.common, &Checkouts::LIST.map(OsStr::new))).map(WorktreeListing::from)
        })
    }

    pub fn checkouts(&self, host: &impl Inspection) -> &Result<Checkouts, Kept> {
        self.checkouts.get_or_init(|| {
            let Ok(checkouts) = self.worktrees(host).as_ref().map_err(Kept::clone)?.as_ref().parse();
            Ok(checkouts)
        })
    }

    pub fn remote_refs_now(&self, host: &impl Inspection) -> Result<RefList, Kept> {
        let listed = host.git(&GitCall::common(&self.common, &[OsStr::new(Self::REMOTE_REFS), self.remote.as_ref()]));
        listed.map_err(|kept| kept.recast(Kept::Unreachable))?.parse()
    }

    pub fn remote_refs(&self, host: &impl Inspection) -> &Result<RefList, Kept> {
        self.remote_refs.get_or_init(|| self.remote_refs_now(host))
    }

    pub fn enclosed(&self, path: &Path) -> Option<()> {
        (self.roots.iter().any(|root| path.starts_with(root) && path != root)
            && !self.checkout.starts_with(path)
            && !self.common.starts_with(path))
        .then_some(())
    }

    pub fn lenders(&self, host: &impl Inspection) -> Vec<Alternates> {
        let linked = self.common.join(Self::LINKED);
        self.roots.iter().chain([&linked]).flat_map(|root| host.neighbours(root)).collect()
    }

    pub fn common(&self) -> &Path {
        &self.common
    }
}
