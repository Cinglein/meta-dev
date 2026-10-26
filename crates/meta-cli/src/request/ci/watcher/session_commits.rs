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
        let start = SessionStart::for_session(session).read();
        let created = start.map_or(Some(Vec::new()), |start| project.created_after(&start));
        let created = created.ok_or_else(|| Listed::new(Self::REFLOG_OFF))?;
        let remote = created.first().filter(|_| reach == Reach::Remote).map(|_| project.remote_heads());
        let heads = remote.transpose()?.unwrap_or_default();
        let unpushed_head = created.first().and_then(|_| project.head()).filter(|head| {
            created.contains(head) && project.pushed_tips(std::slice::from_ref(head), &heads).is_empty()
        });
        Ok(Self { pushed: project.pushed_tips(&created, &heads), unpushed_head })
    }

    pub fn pushed(&self) -> &[Commit] {
        &self.pushed
    }

    pub fn unpushed_head(&self) -> Option<&Commit> {
        self.unpushed_head.as_ref()
    }
}
