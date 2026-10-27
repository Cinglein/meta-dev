use std::panic::resume_unwind;

use super::reach::Reach;
use crate::request::ci::{Commit, SessionId, SessionStart};
use crate::request::output::Listed;
use crate::request::project::Project;

#[derive(Debug)]
pub struct SessionCommits {
    pushed: Vec<Commit>,
    unpushed_head: Option<Commit>,
}

impl SessionCommits {
    const REFLOG_OFF: &'static str = "reflog is off or unreadable here, so this session's commits are unknown: set core.logAllRefUpdates=true; or ask the owner";

    pub fn judged(project: &Project, session: &SessionId, reach: Reach) -> Result<Self, Listed> {
        std::thread::scope(|scope| {
            let head = scope.spawn(|| project.head());
            let start = SessionStart::for_session(session).read();
            let created = start.map_or(Some(Vec::new()), |start| project.created_after(&start));
            let created = created.ok_or_else(|| Listed::new(Self::REFLOG_OFF))?;
            let remote = created.first().filter(|_| reach == Reach::Remote).map(|_| project.remote_heads());
            let heads = remote.transpose()?.unwrap_or_default();
            let pushed = project.pushed_among(&created, &heads);
            let head = head.join().unwrap_or_else(|panic| resume_unwind(panic));
            let unpushed_head =
                created.first().and(head).filter(|head| created.contains(head) && !pushed.contains(head));
            Ok(Self { pushed: project.independent_tips(pushed), unpushed_head })
        })
    }

    pub fn pushed(&self) -> &[Commit] {
        &self.pushed
    }

    pub fn unpushed_head(&self) -> Option<&Commit> {
        self.unpushed_head.as_ref()
    }
}
