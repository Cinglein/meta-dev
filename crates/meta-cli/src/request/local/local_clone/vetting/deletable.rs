use std::path::{Path, PathBuf};

use meta_run::Budget;

use super::asked::{Moved, Stepped, Unbranched};
use super::cause::Cause;
use super::context::Context;
use super::disk::Disk;
use super::entry_kind::EntryKind;
use super::kept::Kept;
use super::repo::Repo;
use super::thaw::Thaw;
use super::vetted::{Link, LocalRefs, Scope, Settled, Step, Vetted};
use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::quarantine::{Quarantine, Stamp};
use crate::request::local::recorded_worktree::RecordedWorktree;

#[derive(Debug)]
pub struct Deletable(Vetted);

impl Deletable {
    pub fn confirmed(first: Vetted, again: Result<Vetted, Kept>) -> Result<Self, Kept> {
        (again? == first).then_some(Self(first)).ok_or(Kept::Changed)
    }

    pub fn removed(self, worktree: &RecordedWorktree, context: &Context, stamp: &Stamp) -> Result<ByteCount, Kept> {
        let link = self.0.link();
        let common = context.repo();
        Disk::occupancy(&[link.path().to_path_buf()], context.budget()).vacant(link.path())?;
        let quarantine = Quarantine::named(link.path(), stamp);
        (Disk::entry(&quarantine) == EntryKind::Missing).then_some(()).ok_or(Kept::Changed)?;
        common.perform(&Moved::new(link.path(), &quarantine)).map_err(|kept| kept.recast(Kept::Unremoved))?;
        let held = link.relocated(&quarantine);
        let arrived = (Disk::entry(link.path()) == EntryKind::Missing).then_some(()).ok_or(Kept::Changed);
        let proved = arrived.and_then(|()| Link::observed(&quarantine, worktree.worktree(), context));
        self.quarantined(&held, proved, context.budget()).map_err(|kept| Self::restored(link, &held, kept, &common))?;
        let _unbranched = common.perform(&Unbranched::new(self.0.branch()));
        Ok(self.0.bytes())
    }

    pub fn unstranded(quarantine: &Path, common: &Path, budget: Budget) -> Result<PathBuf, Kept> {
        let ledger = Disk::text(&Thaw::ledger_of(quarantine, common)).ok_or(Kept::NotFound)?;
        let thaw: Thaw = serde_json::from_str(&ledger).map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        Disk::thawed(&thaw);
        let repo = Repo::common(common, budget);
        let _unlocked = repo.perform(&Stepped::new(Step::Unlock, quarantine));
        repo.perform(&Moved::new(quarantine, thaw.origin())).map(|()| thaw.origin().to_path_buf())
    }

    fn quarantined(&self, held: &Link, proved: Result<Link, Kept>, budget: Budget) -> Result<(), Kept> {
        (proved? == *held).then_some(()).ok_or(Kept::Changed)?;
        let common = held.repo(Scope::Common, budget);
        common.perform(&Stepped::new(Step::Lock, held.path()))?;
        let thaw = Disk::frozen(held.path(), held.thaw_plan(self.0.link().path()))?;
        let quarantine = held.path().to_path_buf();
        let settled = Settled::observed(held, budget);
        let refs = LocalRefs::observed(held, self.0.branch().name(), budget);
        let removed = self
            .0
            .rechecked(&settled, refs)
            .and_then(|()| Disk::occupancy(std::slice::from_ref(&quarantine), budget).vacant(&quarantine))
            .and_then(|()| common.perform(&Stepped::new(Step::Unlock, held.path())))
            .and_then(|()| common.perform(&Stepped::new(Step::Remove, held.path())));
        Disk::thawed(&thaw);
        removed
    }

    fn restored(link: &Link, held: &Link, kept: Kept, common: &Repo) -> Kept {
        let _unlocked = common.perform(&Stepped::new(Step::Unlock, held.path()));
        let kept = kept.recast(Kept::Unremoved);
        common.perform(&Moved::new(held.path(), link.path())).map_or_else(|failure| failure.recast(Kept::Unremoved), |()| kept)
    }
}
