use std::fmt;

use meta_log::LogStore;
use meta_pull_query::LiveData;
use meta_pulls::PullState;
use meta_schema::{Actor, Event, Id, PullRequest, Task, TaskRef};
use meta_state::{State, TaskState};

use crate::request::cli_failure::CliFailure;
use crate::request::output::Listed;
use crate::request::writing::Writing;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloseOut {
    Done(Id<Task>, Actor),
    Untracked(PullRequest),
    Finished(Id<Task>),
    Unheld(Id<Task>),
    Open(Id<Task>, PullRequest),
}

impl CloseOut {
    pub fn judged(state: &State, live: &LiveData, landed: PullRequest) -> Self {
        let linked = state.tasks().iter().find(|task| task.pull_requests().contains(&landed));
        linked.map_or(Self::Untracked(landed), |task| Self::held(state, live, task))
    }

    pub fn entered(self, log: &impl LogStore) -> Result<Listed, CliFailure> {
        match self {
            Self::Done(task, holder) => Writing::new(log, holder, Event::TaskDone(TaskRef::new(task)))
                .entered()
                .map(|written| Listed::new(format_args!("{written}"))),
            kept @ (Self::Untracked(_) | Self::Finished(_) | Self::Unheld(_) | Self::Open(..)) => {
                Ok(Listed::new(format_args!("{kept}\n")))
            }
        }
    }

    fn held(state: &State, live: &LiveData, task: &TaskState) -> Self {
        let id = task.new_task().task();
        let merged = |pull_request: &&PullRequest| {
            live.pull(**pull_request).is_some_and(|pull| pull.state() == PullState::Merged)
        };
        let open = task.pull_requests().iter().find(|pull_request| !merged(pull_request));
        match (task.progress().done(), open, state.holders(id).into_iter().next()) {
            (Some(_), ..) => Self::Finished(id),
            (None, Some(pull_request), _) => Self::Open(id, *pull_request),
            (None, None, None) => Self::Unheld(id),
            (None, None, Some(holder)) => Self::Done(id, holder),
        }
    }
}

impl fmt::Display for CloseOut {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Done(task, holder) => write!(formatter, "{task} done as {holder}"),
            Self::Untracked(pull_request) => write!(formatter, "{pull_request} links no task: nothing to close"),
            Self::Finished(task) => write!(formatter, "{task} already done"),
            Self::Unheld(task) => write!(formatter, "{task} has no holder: left open"),
            Self::Open(task, pull_request) => {
                write!(formatter, "{task} still links unmerged {pull_request}: left open")
            }
        }
    }
}
