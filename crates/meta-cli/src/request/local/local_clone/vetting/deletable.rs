use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::cause::Cause;
use super::context::Context;
use super::entry_kind::EntryKind;
use super::git_call::GitCall;
use super::inspection::Inspection;
use super::kept::Kept;
use super::thaw::Thaw;
use super::vetted::{Link, Step, Vetted};
use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::quarantine::Quarantine;
use crate::request::local::recorded_worktree::RecordedWorktree;

#[derive(Debug)]
pub struct Deletable(Vetted);

impl Deletable {
    const UNLOCK: [&'static str; 2] = ["worktree", "unlock"];
    const MOVE: [&'static str; 2] = ["worktree", "move"];

    pub fn confirmed(first: Vetted, again: Result<Vetted, Kept>) -> Result<Self, Kept> {
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

    pub fn unstranded(quarantine: &Path, common: &Path, host: &impl Inspection) -> Result<PathBuf, Kept> {
        let ledger = host.text(&Thaw::ledger_of(quarantine, common)).ok_or(Kept::NotFound)?;
        let thaw: Thaw = serde_json::from_str(&ledger).map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        host.thawed(&thaw);
        let unlock = [Self::UNLOCK.map(OsStr::new).as_slice(), &[quarantine.as_os_str()]].concat();
        let _unlocked = host.git(&GitCall::common(common, &unlock));
        let back =
            [Self::MOVE.map(OsStr::new).as_slice(), &[quarantine.as_os_str(), thaw.origin().as_os_str()]].concat();
        host.git(&GitCall::common(common, &back)).map(|_| thaw.origin().to_path_buf())
    }

    fn quarantined(&self, held: &Link, proved: Result<Link, Kept>, host: &impl Inspection) -> Result<(), Kept> {
        (proved? == *held).then_some(()).ok_or(Kept::Changed)?;
        host.git(&held.step(Step::Lock))?;
        let thaw = host.frozen(held.path(), held.thaw_plan(self.0.link().path()))?;
        let quarantine = held.path().to_path_buf();
        let removed = self
            .0
            .rechecked(held, host)
            .and_then(|()| host.occupancy(std::slice::from_ref(&quarantine)).vacant(&quarantine))
            .and_then(|()| host.git(&held.step(Step::Unlock)))
            .and_then(|_| host.git(&held.step(Step::Remove)))
            .map(drop);
        host.thawed(&thaw);
        removed
    }

    fn restored(link: &Link, held: &Link, kept: Kept, host: &impl Inspection) -> Kept {
        let _unlocked = host.git(&held.step(Step::Unlock));
        let kept = kept.recast(Kept::Unremoved);
        host.git(&held.moving(link.path())).map_or_else(|failure| failure.recast(Kept::Unremoved), |_| kept)
    }
}
