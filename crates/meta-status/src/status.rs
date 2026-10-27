mod effort_spend;
mod tree;

use std::fmt;

use meta_pull_query::LiveQuery;
use meta_schema::{DueTime, Id, PullRequest, Task};
use meta_state::{Stalling, State, TaskState};

use crate::ending::Ending;
use crate::flag::Flag;
use crate::flags::Flags;
use crate::github::Github;
use crate::lifetime::Lifetime;
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
    pub fn linked_query(state: &State, tasks: &[Id<Task>]) -> LiveQuery {
        let linked = state.tasks().iter().filter(|task| tasks.contains(&task.new_task().task()));
        LiveQuery::new(linked.flat_map(|task| task.pull_requests().to_vec()).collect())
    }

    #[must_use]
    pub fn flags(&self) -> Flags {
        let linked: Vec<PullRequest> =
            self.state.tasks().iter().flat_map(|task| task.pull_requests().to_vec()).collect();
        let orphans = self.github.orphans(&linked).into_iter().map(Flag::Orphan);
        let tasks = self.shown().flat_map(|task| self.task_flags(task));
        let github = self.github.flag().into_iter().chain(self.github.trunk());
        Flags::new(github.chain(tasks).chain(orphans).collect())
    }

    #[must_use]
    pub fn lifetime(&self, task: Id<Task>) -> Lifetime {
        let found = self.state.tasks().iter().find(|known| known.new_task().task() == task);
        let flags = found.into_iter().flat_map(|known| self.task_flags(known));
        let stalled = flags.filter_map(|flag| flag.stalled()).map(|due| Lifetime::Ended(Ending::Stalled(due))).next();
        let pulls = found.into_iter().flat_map(TaskState::pull_requests);
        let lifetimes: Vec<Lifetime> =
            pulls.map(|pull_request| self.github.sight(*pull_request).lifetime(*pull_request)).collect();
        found.map_or(Lifetime::Unlogged, |_| stalled.unwrap_or_else(|| Lifetime::of_pulls(&lifetimes)))
    }

    fn active(state: &State) -> impl Iterator<Item = &TaskState> {
        state.tasks().iter().filter(|task| state.closed(task.entry()).is_none())
    }

    fn shown(&self) -> impl Iterator<Item = &'view TaskState> {
        let state = self.state;
        let status = *self;
        state.tasks().iter().filter(move |task| state.closed(task.entry()).is_none() || !status.pulls(task).is_empty())
    }

    fn pulls(&self, task: &TaskState) -> Vec<PullRequest> {
        let closed = self.state.closed(task.entry());
        let shown = |pull_request: &PullRequest| closed.is_none() || self.github.sight(*pull_request).open().is_some();
        task.pull_requests().iter().copied().filter(shown).collect()
    }

    fn task_flags(&self, task: &TaskState) -> Vec<Flag> {
        let planned = task.new_task();
        let id = planned.task();
        let possible = (self.state.status_of(task).stalling() == Stalling::Possible).then_some(());
        let held = self.state.holders(id).first().map(|_| ());
        let unheld = held.map_or(Some(()), |()| None);
        let overdue = planned.due_time().filter(|due| *due < self.now);
        let stalled = possible.and(unheld).and(overdue).map(|due| Flag::Stalled(id, due));
        let past_due = held.and(overdue).map(|due| Flag::PastDue(id, due));
        let done = held.and(self.github.finished(task.pull_requests())).map(|()| Flag::HeldDone(id));
        let spent = possible.and(planned.effort_allowance()).and(match EffortSpend::measured_for(task) {
            EffortSpend::Unmeasured => None,
        });
        let pulls = self.pulls(task).into_iter();
        let flagged = pulls.filter_map(|pull_request| self.github.sight(pull_request).flag(id, pull_request, unheld));
        stalled.into_iter().chain(past_due).chain(done).chain(spent).chain(flagged).collect()
    }
}

impl fmt::Display for Status<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = self.flags();
        let header = flags.raised().map_or_else(|| "flags: none\n".to_owned(), |flags| format!("flags:\n{flags}"));
        write!(formatter, "{header}{}", Tree::new(*self))
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod lifetime_tests;

#[cfg(test)]
mod done_tests;
