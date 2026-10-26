use super::entry_kind::EntryKind;
use super::inspection::Inspection;
use super::kept::Kept;
use super::occupancy::Occupancy;
use super::vetted::Vetted;
use crate::request::local::byte_count::ByteCount;

#[derive(Debug)]
pub struct Deletable(Vetted);

impl Deletable {
    pub fn confirmed(first: Vetted, occupancy: &Occupancy, again: Result<Vetted, Kept>) -> Result<Self, Kept> {
        occupancy.vacant(first.link().path())?;
        (again? == first).then_some(Self(first)).ok_or(Kept::Changed)
    }

    pub fn removed(self, host: &impl Inspection) -> Result<ByteCount, Kept> {
        let link = self.0.link();
        let quarantine = link.quarantine();
        (host.entry(&quarantine) == EntryKind::Missing).then_some(()).ok_or(Kept::Changed)?;
        host.git(&link.moving(&quarantine)).map_err(|kept| kept.recast(Kept::Unremoved))?;
        let held = link.relocated(&quarantine);
        let removed = self.0.rechecked(&quarantine, host).and_then(|()| host.git(&held.removal()));
        removed.map_err(|kept| {
            let kept = kept.recast(Kept::Unremoved);
            host.git(&held.moving(link.path())).map_or_else(|failure| failure.recast(Kept::Unremoved), |_| kept)
        })?;
        let _unbranched = host.git(&link.unbranch(self.0.branch()));
        Ok(self.0.bytes())
    }
}
