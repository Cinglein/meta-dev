use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::cause::Cause;
use super::context::Context;
use super::entry_kind::EntryKind;
use super::git_call::GitCall;
use super::halt::Halt;
use super::kept::Kept;
use super::look::Look;
use super::replay::Replay;
use super::seen::Seen;
use super::thaw::Thaw;
use super::vetted::{Link, Step, Vetted};
use crate::request::local::byte_count::ByteCount;
use crate::request::local::local_clone::quarantine::{Quarantine, Stamp};
use crate::request::local::recorded_worktree::RecordedWorktree;

#[derive(Debug)]
pub struct Deletable(Vetted);

impl Deletable {
    const UNLOCK: [&'static str; 2] = ["worktree", "unlock"];
    const MOVE: [&'static str; 2] = ["worktree", "move"];

    pub fn confirmed(first: Vetted, again: Result<Vetted, Kept>) -> Result<Self, Kept> {
        (again? == first).then_some(Self(first)).ok_or(Kept::Changed)
    }

    // Each Destroy below comes straight after the Look it depends on; the interpreter refuses it otherwise.
    pub fn removed(
        self,
        worktree: &RecordedWorktree,
        context: &Context,
        stamp: &Stamp,
        replay: &Replay<'_>,
    ) -> Result<ByteCount, Halt> {
        let link = self.0.link();
        let quarantine = Quarantine::named(link.path(), stamp);
        let looks = [Look::Occupancy(vec![link.path().to_path_buf()]), Look::Entry(quarantine.clone())];
        let [open, taken] = <[Seen; 2]>::try_from(replay.looked(looks.to_vec())?).map_err(|_| Replay::diverged())?;
        open.occupancy()?.vacant(link.path())?;
        (taken.entry()? == EntryKind::Missing).then_some(()).ok_or(Kept::Changed)?;
        replay.destroyed(link.moving(&quarantine))?.map_err(|kept| kept.recast(Kept::Unremoved))?;
        let held = link.relocated(&quarantine);
        let arrived = replay.entry(link.path()).and_then(|entry| {
            (entry == EntryKind::Missing).then_some(()).ok_or(Halt::Kept(Kept::Changed))
        });
        let proved = Halt::settled(arrived.and_then(|()| Link::proved(&quarantine, worktree.worktree(), context, replay)))?;
        match Halt::settled(self.quarantined(&held, proved, replay))? {
            Ok(()) => {}
            Err(kept) => return Err(Halt::Kept(Self::restored(link, &held, kept, replay)?)),
        }
        let _unbranched = replay.changed(link.unbranch(self.0.branch()))?;
        Ok(self.0.bytes())
    }

    pub fn unstranded(quarantine: &Path, common: &Path, replay: &Replay<'_>) -> Result<PathBuf, Halt> {
        let ledger = replay.text(&Thaw::ledger_of(quarantine, common))?.ok_or(Kept::NotFound)?;
        let thaw: Thaw = serde_json::from_str(&ledger).map_err(|failure| Kept::Uninspected(Cause::new(failure)))?;
        replay.thawed(&thaw)?;
        let unlock = [Self::UNLOCK.map(OsStr::new).as_slice(), &[quarantine.as_os_str()]].concat();
        let _unlocked = replay.changed(GitCall::common(common, &unlock))?;
        let back =
            [Self::MOVE.map(OsStr::new).as_slice(), &[quarantine.as_os_str(), thaw.origin().as_os_str()]].concat();
        replay.changed(GitCall::common(common, &back))??;
        Ok(thaw.origin().to_path_buf())
    }

    fn quarantined(&self, held: &Link, proved: Result<Link, Kept>, replay: &Replay<'_>) -> Result<(), Halt> {
        (proved? == *held).then_some(()).ok_or(Kept::Changed)?;
        replay.changed(held.step(Step::Lock))??;
        let thaw = replay.frozen(held.path(), held.thaw_plan(self.0.link().path()))??;
        let removed = Halt::settled(self.emptied(held, replay))?;
        replay.thawed(&thaw)?;
        Ok(removed?)
    }

    // Unlock first, so that the re-read and the scan of open files are the last turns before the removal.
    fn emptied(&self, held: &Link, replay: &Replay<'_>) -> Result<(), Halt> {
        replay.changed(held.step(Step::Unlock))??;
        self.0.rechecked(held, replay)?;
        let quarantine = held.path().to_path_buf();
        replay.occupancy(std::slice::from_ref(&quarantine))?.vacant(&quarantine)?;
        replay.destroyed(held.step(Step::Remove))??;
        Ok(())
    }

    fn restored(link: &Link, held: &Link, kept: Kept, replay: &Replay<'_>) -> Result<Kept, Halt> {
        let _unlocked = replay.changed(held.step(Step::Unlock))?;
        let kept = kept.recast(Kept::Unremoved);
        Ok(replay.changed(held.moving(link.path()))?.map_or_else(|failure| failure.recast(Kept::Unremoved), |_| kept))
    }
}
