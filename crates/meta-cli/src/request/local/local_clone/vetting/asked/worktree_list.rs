use crate::request::local::local_clone::vetting::kept::Kept;
use crate::request::local::local_clone::vetting::query::Query;
use crate::request::local::local_clone::vetting::vet_argv::VetArgv;
use crate::request::local::local_clone::worktree_listing::WorktreeListing;

#[derive(Clone, Copy, Debug)]
pub struct WorktreeList;

impl Query for WorktreeList {
    type View = WorktreeListing;
    const ARGV: VetArgv = VetArgv::WorktreeList;

    fn viewed(&self, out: &str) -> Result<WorktreeListing, Kept> {
        Ok(WorktreeListing::from(out.to_owned()))
    }
}
