use super::Vetted;
use super::checkouts::Checkouts;
use super::link::{Guard, Pointers};
use super::local_refs::LocalRefs;
use super::placed::Placed;
use super::ref_list::RefList;
use super::settled::Settled;
use crate::request::local::local_clone::vetting::context::Context;
use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::recorded_worktree::RecordedWorktree;

#[derive(Clone, Debug)]
pub struct Survey {
    pub placed: Placed,
    pub pointers: Pointers,
    pub guard: Guard,
    pub checkouts: Result<Checkouts, Kept>,
    pub local: Result<LocalRefs, Kept>,
    pub remote: Result<RefList, Kept>,
    pub beyond: Result<(), Kept>,
    pub again: Option<Result<LocalRefs, Kept>>,
    pub settled: Settled,
}

impl Survey {
    pub fn judged(&self, worktree: &RecordedWorktree, context: &Context) -> Result<Vetted, Kept> {
        let path = worktree.path();
        self.placed.judged(path, context)?;
        let link = self.pointers.proved(path, worktree.worktree(), context.common())?;
        self.guard.judged()?;
        let branch = worktree.branch().qualified();
        self.checkouts.clone()?.held(&branch, link.path())?;
        let refs = self.local.clone()?.refs()?;
        refs.matched(&self.remote.clone()?)?;
        self.beyond.clone()?;
        let again = self.again.clone().map(|again| again?.refs());
        again.map_or(Ok(()), |again| (again? == refs).then_some(()).ok_or(Kept::Changed))?;
        let bytes = self.settled.judged()?;
        let named = refs.named(&branch).cloned().ok_or(Kept::NoBranch)?;
        Ok(Vetted::new(link, refs, named, bytes))
    }
}
