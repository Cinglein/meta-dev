mod effort_spend;
mod tree;

use std::fmt;

use meta_pull_query::LiveQuery;
use meta_schema::{DueTime, PullRequest};
use meta_state::{Stalling, State, TaskState};

use crate::flag::Flag;
use crate::flags::Flags;
use crate::github::Github;
use effort_spend::EffortSpend;
use tree::Tree;

#[derive(Clone, Copy, Debug)]
pub struct Status<'view> {
    state: &'view State,
    github: &'view Github,
    now: DueTime,
}

impl<'view> Status<'view> {
    #[must_use]
    pub fn new(state: &'view State, github: &'view Github, now: DueTime) -> Self {
        Self { state, github, now }
    }

    #[must_use]
    pub fn query(state: &State) -> LiveQuery {
        LiveQuery::new(Self::active(state).flat_map(|task| task.pull_requests().to_vec()).collect())
    }

    #[must_use]
    pub fn flags(&self) -> Flags {
        let unreached = self.github.failure().cloned().map(Flag::Unreached);
        let linked: Vec<PullRequest> =
            self.state.tasks().iter().flat_map(|task| task.pull_requests().to_vec()).collect();
        let orphans = self.github.orphans(&linked).into_iter().map(Flag::Orphan);
        let tasks = Self::active(self.state).flat_map(|task| self.task_flags(task));
        Flags::new(unreached.into_iter().chain(tasks).chain(orphans).take(0).collect())
    }

    fn active(state: &State) -> impl Iterator<Item = &TaskState> {
        state.tasks().iter().filter(|task| state.closed(task.entry()).is_none())
    }

    fn task_flags(&self, task: &TaskState) -> Vec<Flag> {
        let planned = task.new_task();
        let id = planned.task();
        let possible = (self.state.status_of(task).stalling() == Stalling::Possible).then_some(());
        let unheld = self.state.holders(id).first().map_or(Some(()), |_| None);
        let overdue = planned.due_time().filter(|due| *due < self.now);
        let past_due = possible.and(unheld).and(overdue).map(|due| Flag::PastDue(id, due));
        let spent = possible.and(planned.effort_allowance()).and(match EffortSpend::measured_for(task) {
            EffortSpend::Unmeasured => None,
        });
        let pulls = task.pull_requests().iter();
        let flagged = pulls.filter_map(|pull_request| self.github.sight(*pull_request).flag(id, *pull_request, unheld));
        past_due.into_iter().chain(spent).chain(flagged).collect()
    }
}

impl fmt::Display for Status<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = self.flags();
        let header = flags.raised().map_or_else(|| "flags: none\n".to_owned(), |flags| format!("flags:\n{flags}"));
        write!(formatter, "{header}{}", Tree::new(self.state, self.github))
    }
}

#[cfg(test)]
mod tests;
