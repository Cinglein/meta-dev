mod checkouts;
mod index;
mod link;
mod local_refs;
mod pass;
mod placed;
mod porcelain;
mod ref_line;
mod ref_list;
mod settled;
mod survey;
mod tree;

use super::asked::Beyond;
use super::context::Context;
use super::kept::Kept;
use crate::request::local::byte_count::ByteCount;
use crate::request::local::recorded_worktree::RecordedWorktree;
pub use checkouts::Checkouts;
pub use index::Index;
pub use link::{Guard, Link, Pointers, Resolved, Scope, Step};
pub use local_refs::LocalRefs;
pub use pass::Pass;
pub use placed::Placed;
pub use porcelain::Porcelain;
pub use ref_line::RefLine;
pub use ref_list::RefList;
pub use settled::Settled;
pub use survey::Survey;
#[cfg(test)]
pub use tree::Tree;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vetted {
    link: Link,
    refs: RefList,
    branch: RefLine,
    bytes: ByteCount,
}

impl Vetted {
    pub fn new(link: Link, refs: RefList, branch: RefLine, bytes: ByteCount) -> Self {
        Self { link, refs, branch, bytes }
    }

    pub fn proved(worktree: &RecordedWorktree, context: &Context, pass: Pass) -> Result<Self, Kept> {
        let path = worktree.path();
        let budget = context.budget();
        let placed = Placed::observed(path);
        placed.judged(path, context)?;
        let pointers = Pointers::observed(path);
        let link = pointers.proved(path, worktree.worktree(), context.common())?;
        let guard = Guard::observed(&link, budget);
        guard.judged()?;
        let branch = worktree.branch().qualified();
        let checkouts = match pass {
            Pass::Swept => context.checkouts(),
            Pass::Fresh => context.checkouts_now(),
        };
        checkouts.clone()?.held(&branch, link.path())?;
        let local = LocalRefs::observed(&link, &branch, budget);
        let refs = local.clone()?.refs()?;
        let remote = match pass {
            Pass::Swept => context.remote_refs(),
            Pass::Fresh => context.remote_refs_now(),
        };
        refs.matched(&remote.clone()?)?;
        let beyond = Beyond::new(refs.objects(), remote.clone()?.objects());
        let beyond = link.repo(Scope::Common, budget).observe(&beyond);
        beyond.clone()?;
        let again = (pass == Pass::Fresh).then(|| LocalRefs::observed(&link, &branch, budget));
        let settled = Settled::observed(&link, budget);
        let survey = Survey { placed, pointers, guard, checkouts, local, remote, beyond, again, settled };
        survey.judged(worktree, context)
    }

    pub fn rechecked(&self, settled: &Settled, refs: Result<LocalRefs, Kept>) -> Result<(), Kept> {
        (settled.judged()? == self.bytes && refs?.refs()? == self.refs).then_some(()).ok_or(Kept::Changed)
    }

    pub fn link(&self) -> &Link {
        &self.link
    }

    pub fn branch(&self) -> &RefLine {
        &self.branch
    }

    pub fn bytes(&self) -> ByteCount {
        self.bytes
    }
}
