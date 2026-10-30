use std::time::Duration;

use chrono::{DateTime, Utc};
use meta_schema::{Block, Blocker, ExternalKind, Id, PullRequest, Task};

use crate::flag::Flag;
use crate::github::Github;
use crate::no_run::NoRun;

#[derive(Clone, Copy, Debug)]
pub struct Unrunning<'github> {
    github: &'github Github,
    now: DateTime<Utc>,
    within: Duration,
}

impl<'github> Unrunning<'github> {
    #[must_use]
    pub fn new(github: &'github Github, now: DateTime<Utc>, within: Duration) -> Self {
        Self { github, now, within }
    }

    #[must_use]
    pub fn flags(&self, task: Id<Task>, block: &Block, pulls: &[PullRequest]) -> Vec<Flag> {
        let on_ci = Some(block).filter(|block| block.blocker() == Blocker::External(ExternalKind::Ci));
        on_ci.iter().flat_map(|block| pulls.iter().filter_map(|pull| self.flag(task, block, *pull))).collect()
    }

    fn flag(&self, task: Id<Task>, block: &Block, pull: PullRequest) -> Option<Flag> {
        let unrun = self.github.sight(pull).unrun(self.now, self.within)?.waited_at(block.head())?;
        Some(Flag::Unrun(task, pull, NoRun::new(unrun)))
    }
}
