use std::time::Instant;

use meta_log::RemoteHeads;

use super::reach::Reach;
use crate::request::checkout::Checkout;
use crate::request::ci::{Commit, SessionId, SessionStart};
use crate::request::output::Listed;
use crate::request::project::Project;

#[derive(Debug, PartialEq, Eq)]
pub struct SessionCommits {
    pushed: Vec<Commit>,
    unpushed_head: Option<Commit>,
    checkout: Checkout,
    heads: RemoteHeads,
}

impl SessionCommits {
    const REFLOG_OFF: &'static str = "reflog is off or unreadable here, so this session's commits are unknown: set core.logAllRefUpdates=true; or ask the owner";

    pub fn judged(project: &Project, session: &SessionId, reach: Reach, deadline: Instant) -> Result<Self, Listed> {
        let start = SessionStart::for_session(session).read();
        let created = start.map_or(Some(Vec::new()), |start| project.created_after(&start, deadline));
        let created = Self::known(created)?;
        let remote = reach.asks_remote(&created).map(|()| project.remote_heads(deadline));
        let heads = remote.transpose()?.unwrap_or_default();
        let unpushed = project.unpushed(&created, &heads.tips(), deadline);
        let seen = Self::new(&created, unpushed, project.checkout().clone(), heads);
        let independent = project.independent_tips(seen.pushed(), deadline);
        Ok(seen.tipped(independent))
    }

    pub fn known(created: Option<Vec<Commit>>) -> Result<Vec<Commit>, Listed> {
        created.ok_or_else(|| Listed::new(Self::REFLOG_OFF))
    }

    pub fn new(created: &[Commit], unpushed: Option<Vec<Commit>>, checkout: Checkout, heads: RemoteHeads) -> Self {
        let pushed: Vec<Commit> = unpushed.map_or_else(Vec::new, |unpushed| {
            created.iter().filter(|commit| !unpushed.contains(commit)).cloned().collect()
        });
        let unpushed_head =
            created.first().and(checkout.head()).filter(|head| created.contains(head) && !pushed.contains(head));
        Self { unpushed_head: unpushed_head.cloned(), pushed, checkout, heads }
    }

    #[must_use]
    pub fn tipped(self, independent: Option<Vec<Commit>>) -> Self {
        Self { pushed: independent.unwrap_or(self.pushed), ..self }
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

#[cfg(test)]
mod tests;
