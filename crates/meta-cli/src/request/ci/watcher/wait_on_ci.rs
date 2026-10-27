use std::fmt;

use meta_log::LogStore as _;
use meta_schema::{CommitId, ExternalKind, Id, Task};
use meta_state::{State, TaskStatus};

use crate::request::output::Listed;
use crate::request::project::Project;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitOnCi {
    Recorded,
    Unrecorded(Id<Task>),
    Waiting(Id<Task>, TaskStatus),
    Done(Id<Task>),
    Taskless,
}

impl WaitOnCi {
    const ON_CI: TaskStatus = TaskStatus::WaitingOnExternal(ExternalKind::Ci);
    const READY: [TaskStatus; 2] = [TaskStatus::Open, TaskStatus::Assigned];

    pub fn here(project: &Project) -> Self {
        let state = project.log().ok().and_then(|log| log.read().ok()).map(|read| State::fold(read.records()));
        let head = project.head().and_then(|head| head.commit_id());
        let task = project.unprotected_branch().and_then(|branch| branch.task());
        state.map_or(Self::Taskless, |state| Self::in_state(&state, head.as_ref(), task))
    }

    fn in_state(state: &State, head: Option<&CommitId>, task: Option<Id<Task>>) -> Self {
        let covered = state.tasks().iter().find(|known| {
            state.status_of(known) == Self::ON_CI && head.is_some_and(|head| known.progress().waited_at() == Some(head))
        });
        covered.map_or_else(|| task.map_or(Self::Taskless, |task| Self::of_task(state, task)), |_| Self::Recorded)
    }

    fn of_task(state: &State, task: Id<Task>) -> Self {
        let known = state.tasks().iter().find(|known| known.new_task().task() == task);
        known.map_or(Self::Taskless, |known| {
            let status = state.status_of(known);
            let retired = (status == TaskStatus::Retired).then_some(Self::Taskless);
            let ready = Self::READY.contains(&status).then_some(Self::Unrecorded(task));
            let done = known.progress().done().map(|_| Self::Done(task));
            retired.or(ready).or(done).unwrap_or(Self::Waiting(task, status))
        })
    }

    pub fn stop_reason(self, unresolved: &str) -> Option<Listed> {
        match self {
            Self::Recorded => None,
            Self::Unrecorded(_) | Self::Waiting(..) | Self::Done(_) | Self::Taskless => {
                Some(Listed::new(unresolved).then(self))
            }
        }
    }
}

impl fmt::Display for WaitOnCi {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recorded => Ok(()),
            Self::Unrecorded(task) => write!(
                formatter,
                "; or run `meta wait {task} --on ci` here (`meta claim {task}` first if you do not hold it)"
            ),
            Self::Waiting(task, status) => write!(
                formatter,
                "; or task {task} is {status}: run `meta resume {task}` first, then `meta wait {task} --on ci` here"
            ),
            Self::Done(task) => write!(
                formatter,
                "; or task {task} is done: run `meta reopen {task}` and `meta claim {task}`, or open a new task, then `meta wait <task> --on ci` here"
            ),
            Self::Taskless => formatter.write_str(
                "; or open or claim a task (`meta open <text> --division <path>`, `meta claim <task>`), then run `meta wait <task> --on ci` here",
            ),
        }
    }
}

#[cfg(test)]
mod tests;
