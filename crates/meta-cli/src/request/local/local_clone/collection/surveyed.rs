use std::path::{Path, PathBuf};

use crate::request::local::local_clone::vetting::{
    Alternates, Cause, Checkouts, Context, EntryKind, Halt, Kept, Look, RefList, Replay,
};
use crate::request::local::local_clone::worktree_listing::WorktreeListing;

// What a sweep reads once, in one Look: the worktree list, each recorded path's entry, and,
// only when some worktree may be vetted, the remote refs and the lenders under the roots.
#[derive(Clone, Debug)]
pub struct Surveyed {
    listing: Result<WorktreeListing, Kept>,
    checkouts: Result<Checkouts, Kept>,
    remote: Result<RefList, Kept>,
    lenders: Vec<Alternates>,
    entries: Vec<EntryKind>,
}

impl Surveyed {
    const UNREAD: &'static str = "not read: nothing to vet";

    pub fn read(context: &Context, paths: &[PathBuf], vetting: bool, replay: &Replay<'_>) -> Result<Self, Halt> {
        let wider = vetting.then(|| [[Look::Git(context.remote_refs())].as_slice(), &context.lender_looks()].concat());
        let looks: Vec<Look> = [Look::Git(context.worktrees())]
            .into_iter()
            .chain(paths.iter().map(|path| Look::Entry(path.clone())))
            .chain(wider.into_iter().flatten())
            .collect();
        let mut seen = replay.looked(looks)?.into_iter();
        let mut next = || seen.next().ok_or_else(Replay::diverged);
        let listing = next()?.git()?.map(WorktreeListing::from);
        let entries = paths.iter().map(|_| next()?.entry()).collect::<Result<Vec<_>, Halt>>()?;
        let unread = || Err(Kept::Uninspected(Cause::new(Self::UNREAD)));
        let (remote, lenders) = if vetting {
            let remote = RefList::remote(next()?.git()?);
            let lenders = [next()?.lenders()?, next()?.lenders()?].concat();
            (remote, lenders)
        } else {
            (unread(), Vec::new())
        };
        let checkouts = listing.clone().map(|listing| {
            let Ok(checkouts) = listing.as_ref().parse::<Checkouts>();
            checkouts
        });
        Ok(Self { listing, checkouts, remote, lenders, entries })
    }

    pub fn listing(&self) -> &Result<WorktreeListing, Kept> {
        &self.listing
    }

    pub fn checkouts(&self) -> &Result<Checkouts, Kept> {
        &self.checkouts
    }

    pub fn remote(&self) -> &Result<RefList, Kept> {
        &self.remote
    }

    pub fn lenders(&self) -> Vec<Alternates> {
        self.lenders.clone()
    }

    pub fn entry(&self, at: usize) -> EntryKind {
        self.entries.get(at).copied().unwrap_or(EntryKind::Unreadable)
    }

    pub fn path_listed(&self, checkout: &Path) -> Vec<PathBuf> {
        self.listing.as_ref().map(|listing| listing.paths(checkout)).unwrap_or_default()
    }
}
