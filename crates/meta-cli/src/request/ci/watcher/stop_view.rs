mod stop_reason;
mod watched;

use std::ops::ControlFlow;

use super::session_commits::SessionCommits;
use crate::request::ci::ci_status::CiStatus;
use crate::request::output::Listed;
pub use stop_reason::StopReason;
pub use watched::Watched;

#[derive(Debug)]
pub struct StopView<'seen> {
    seen: &'seen Result<SessionCommits, Listed>,
    unwritable: Option<String>,
    statuses: Vec<Watched>,
}

impl<'seen> StopView<'seen> {
    pub fn new(
        seen: &'seen Result<SessionCommits, Listed>,
        unwritable: Option<String>,
        statuses: Vec<Watched>,
    ) -> Self {
        Self { seen, unwritable, statuses }
    }

    pub fn reason(self) -> ControlFlow<StopReason, Vec<Watched>> {
        let unknown = self.seen.as_ref().err().cloned().map(StopReason::Unknown);
        let pushed = self.seen.as_ref().ok();
        let unpushed = pushed.and_then(SessionCommits::unpushed_head).cloned().map(StopReason::Unpushed);
        let unwatched = self.statuses.iter().find(|watched| watched.status().is_none());
        let unwatched = unwatched.map(|watched| StopReason::Unwatched(watched.commit().clone()));
        let known = self.statuses.iter().find_map(|watched| watched.status().and_then(CiStatus::stop_reason));
        let reason = unknown.or(self.unwritable.map(StopReason::Unwritable)).or(unpushed).or(unwatched);
        reason.or(known.map(StopReason::Known)).map_or(ControlFlow::Continue(self.statuses), ControlFlow::Break)
    }
}

#[cfg(test)]
mod tests;
