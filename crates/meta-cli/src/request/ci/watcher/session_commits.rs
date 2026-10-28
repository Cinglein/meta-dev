mod checkout;

use std::panic::resume_unwind;
use std::time::Instant;

use meta_log::RemoteHeads;

use super::reach::Reach;
use crate::request::ci::{Commit, SessionId, SessionStart};
use crate::request::output::Listed;
use crate::request::project::Project;
use checkout::Checkout;

#[derive(Debug)]
pub struct SessionCommits {
    pushed: Vec<Commit>,
    unpushed_head: Option<Commit>,
    checkout: Checkout,
    heads: RemoteHeads,
}

impl SessionCommits {
    const REFLOG_OFF: &'static str = "reflog is off or unreadable here, so this session's commits are unknown: set core.logAllRefUpdates=true; or ask the owner";

    pub fn judged(project: &Project, session: &SessionId, reach: Reach, deadline: Instant) -> Result<Self, Listed> {
        std::thread::scope(|scope| {
            let checkout = scope.spawn(|| Checkout::read(project, deadline));
            let start = SessionStart::for_session(session).read();
            let created = start.map_or(Some(Vec::new()), |start| project.created_after(&start, deadline));
            let created = created.ok_or_else(|| Listed::new(Self::REFLOG_OFF))?;
            let remote = created.first().filter(|_| reach == Reach::Remote).map(|_| project.remote_heads(deadline));
            let heads = remote.transpose()?.unwrap_or_default();
            let pushed = project.pushed_among(&created, &heads.tips(), deadline);
            let checkout = checkout.join().unwrap_or_else(|panic| resume_unwind(panic));
            let unpushed_head =
                created.first().and(checkout.head()).filter(|head| created.contains(head) && !pushed.contains(head));
            let unpushed_head = unpushed_head.cloned();
            Ok(Self { pushed: project.independent_tips(pushed, deadline), unpushed_head, checkout, heads })
        })
    }

    pub fn pushed(&self) -> &[Commit] {
        &self.pushed
    }

    pub fn unpushed_head(&self) -> Option<&Commit> {
        self.unpushed_head.as_ref()
    }

    pub fn checkout(&self) -> &Checkout {
        &self.checkout
    }

    pub fn heads(&self) -> &RemoteHeads {
        &self.heads
    }
}
