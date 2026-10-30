use std::time::Duration;

use meta_pulls::Patience;
use meta_schema::{Block, Blocker, DueTime, ExternalKind, Id, PullRequest, Task};

use crate::flag::Flag;
use crate::github::Github;

#[derive(Clone, Copy, Debug)]
pub struct Unrunning<'github> {
    github: &'github Github,
    now: DueTime,
    within: Duration,
}

impl<'github> Unrunning<'github> {
    #[must_use]
    pub fn new(github: &'github Github, now: DueTime, within: Duration) -> Self {
        Self { github, now, within }
    }

    #[must_use]
    pub fn flags(&self, task: Id<Task>, block: &Block, since: DueTime, pulls: &[PullRequest]) -> Vec<Flag> {
        let on_ci = Some(block).filter(|block| block.blocker() == Blocker::External(ExternalKind::Ci));
        on_ci.iter().flat_map(|block| pulls.iter().filter_map(|pull| self.flag(task, block, since, *pull))).collect()
    }

    fn flag(&self, task: Id<Task>, block: &Block, since: DueTime, pull: PullRequest) -> Option<Flag> {
        Patience::new(self.now.elapsed_since(since)?, self.within).spent()?;
        let commit = self.github.sight(pull).unrun()?;
        block.head().is_none_or(|head| *head == commit).then_some(())?;
        Some(Flag::Unrun(task, pull, commit, since.after(self.within)?))
    }
}
