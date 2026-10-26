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
        host.git(&link.removal()).map_err(|kept| kept.recast(Kept::Unremoved))?;
        let unbranched = self.0.branch().map(|branch| host.git(&link.unbranch(branch)));
        unbranched.transpose().map_err(|kept| kept.recast(Kept::Unremoved))?;
        Ok(self.0.bytes())
    }
}
