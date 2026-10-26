use super::context::Context;
use super::entry_kind::EntryKind;
use super::inspection::Inspection;
use super::kept::Kept;
use super::occupancy::Occupancy;
use super::vetted::{Link, Step, Vetted};
use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::quarantine::Quarantine;
use crate::request::local::recorded_worktree::RecordedWorktree;

#[derive(Debug)]
pub struct Deletable(Vetted);

impl Deletable {
    pub fn confirmed(first: Vetted, occupancy: &Occupancy, again: Result<Vetted, Kept>) -> Result<Self, Kept> {
        occupancy.vacant(first.link().path())?;
        (again? == first).then_some(Self(first)).ok_or(Kept::Changed)
    }

    pub fn removed(
        self,
        worktree: &RecordedWorktree,
        context: &Context,
        host: &impl Inspection,
    ) -> Result<ByteCount, Kept> {
        let link = self.0.link();
        host.occupancy(&[link.path().to_path_buf()]).vacant(link.path())?;
        let quarantine = Quarantine::named(link.path(), &host.stamp());
        (host.entry(&quarantine) == EntryKind::Missing).then_some(()).ok_or(Kept::Changed)?;
        host.git(&link.moving(&quarantine)).map_err(|kept| kept.recast(Kept::Unremoved))?;
        let held = link.relocated(&quarantine);
        let arrived = (host.entry(link.path()) == EntryKind::Missing).then_some(()).ok_or(Kept::Changed);
        let proved = arrived.and_then(|()| Link::proved(&quarantine, worktree.worktree(), context, host));
        self.quarantined(&held, proved, host).map_err(|kept| Self::restored(link, &held, kept, host))?;
        let _unbranched = host.git(&link.unbranch(self.0.branch()));
        Ok(self.0.bytes())
    }

    fn quarantined(&self, held: &Link, proved: Result<Link, Kept>, host: &impl Inspection) -> Result<(), Kept> {
        (proved? == *held).then_some(()).ok_or(Kept::Changed)?;
        host.git(&held.step(Step::Lock))?;
        self.0.rechecked(held, host)?;
        host.git(&held.step(Step::Unlock))?;
        host.git(&held.step(Step::Remove)).map(drop)
    }

    fn restored(link: &Link, held: &Link, kept: Kept, host: &impl Inspection) -> Kept {
        let _unlocked = host.git(&held.step(Step::Unlock));
        let kept = kept.recast(Kept::Unremoved);
        host.git(&held.moving(link.path())).map_or_else(|failure| failure.recast(Kept::Unremoved), |_| kept)
    }
}
