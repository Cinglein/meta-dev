use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use meta_log::RemoteName;
use meta_run::Budget;

use super::alternates::Alternates;
use super::asked::{RemoteRefs, WorktreeList};
use super::disk::Disk;
use super::kept::Kept;
use super::repo::Repo;
use super::vetted::{Checkouts, RefList};
use crate::request::local::local_clone::worktree_listing::WorktreeListing;

#[derive(Debug)]
pub struct Context {
    checkout: PathBuf,
    common: PathBuf,
    root: PathBuf,
    remote: RemoteName,
    budget: Budget,
    listing: Result<WorktreeListing, Kept>,
    remote_refs: OnceLock<Result<RefList, Kept>>,
    lenders: OnceLock<Vec<Alternates>>,
}

impl Context {
    const LINKED: &'static str = "worktrees";

    pub fn observed(checkout: PathBuf, common: PathBuf, root: PathBuf, remote: RemoteName, budget: Budget) -> Self {
        let listing = Repo::common(&common, budget).observe(&WorktreeList);
        Self::new([checkout, common, root], remote, budget, listing)
    }

    pub fn new(
        places: [PathBuf; 3],
        remote: RemoteName,
        budget: Budget,
        listing: Result<WorktreeListing, Kept>,
    ) -> Self {
        let [checkout, common, root] = places;
        Self {
            checkout,
            common,
            root,
            remote,
            budget,
            listing,
            remote_refs: OnceLock::new(),
            lenders: OnceLock::new(),
        }
    }

    #[cfg(test)]
    pub fn seen(self, remote_refs: Result<RefList, Kept>, lenders: Vec<Alternates>) -> Self {
        let context = Self { remote_refs: OnceLock::from(remote_refs), ..self };
        Self { lenders: OnceLock::from(lenders), ..context }
    }

    pub fn listing(&self) -> &Result<WorktreeListing, Kept> {
        &self.listing
    }

    pub fn checkouts(&self) -> Result<Checkouts, Kept> {
        let Ok(checkouts) = self.listing.as_ref().map_err(Kept::clone)?.as_ref().parse();
        Ok(checkouts)
    }

    pub fn checkouts_now(&self) -> Result<Checkouts, Kept> {
        let listing = self.repo().observe(&WorktreeList)?;
        let Ok(checkouts) = listing.as_ref().parse();
        Ok(checkouts)
    }

    pub fn remote_refs_now(&self) -> Result<RefList, Kept> {
        let listed = self.repo().observe(&RemoteRefs::new(self.remote.clone()));
        listed.map_err(|kept| kept.recast(Kept::Unreachable))
    }

    pub fn remote_refs(&self) -> Result<RefList, Kept> {
        self.remote_refs.get_or_init(|| self.remote_refs_now()).clone()
    }

    pub fn lenders(&self) -> &[Alternates] {
        self.lenders.get_or_init(|| [Disk::lenders(&self.root), Disk::lenders(&self.common.join(Self::LINKED))].concat())
    }

    pub fn enclosed(&self, path: &Path) -> Option<()> {
        (path.starts_with(&self.root)
            && path != self.root
            && !self.checkout.starts_with(path)
            && !self.common.starts_with(path))
        .then_some(())
    }

    pub fn repo(&self) -> Repo {
        Repo::common(&self.common, self.budget)
    }

    pub fn common(&self) -> &Path {
        &self.common
    }

    pub fn budget(&self) -> Budget {
        self.budget
    }
}
