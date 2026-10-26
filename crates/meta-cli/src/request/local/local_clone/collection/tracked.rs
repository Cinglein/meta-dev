use meta_status::Lifetime;

use crate::request::local::recorded_worktree::RecordedWorktree;

#[derive(Clone, Debug)]
pub struct Tracked {
    worktree: RecordedWorktree,
    lifetime: Lifetime,
}

impl Tracked {
    pub fn new(worktree: RecordedWorktree, lifetime: Lifetime) -> Self {
        Self { worktree, lifetime }
    }

    pub fn worktree(&self) -> &RecordedWorktree {
        &self.worktree
    }

    pub fn lifetime(&self) -> Lifetime {
        self.lifetime
    }
}
