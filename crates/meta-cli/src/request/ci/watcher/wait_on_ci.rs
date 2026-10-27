use std::fmt;

use meta_log::LogStore as _;
use meta_schema::{ExternalKind, Id, Task};
use meta_state::{State, TaskStatus};

use crate::request::output::Listed;
use crate::request::project::Project;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitOnCi {
    Recorded,
    Unrecorded(Id<Task>),
    Taskless,
}

impl WaitOnCi {
    const RECORDED: TaskStatus = TaskStatus::WaitingOnExternal(ExternalKind::Ci);

    pub fn of_branch(project: &Project) -> Self {
        let task = project.unprotected_branch().and_then(|branch| branch.task());
        task.map_or(Self::Taskless, |task| {
            let state = project.log().ok().and_then(|log| log.read().ok()).map(|read| State::fold(read.records()));
            state.map_or(Self::Unrecorded(task), |state| Self::in_state(&state, task))
        })
    }

    fn in_state(state: &State, task: Id<Task>) -> Self {
        let waiting = state
            .tasks()
            .iter()
            .find(|known| known.new_task().task() == task && state.status_of(known) == Self::RECORDED);
        waiting.map_or(Self::Unrecorded(task), |_| Self::Recorded)
    }

    pub fn stop_reason(self, unresolved: &str) -> Option<Listed> {
        match self {
            Self::Recorded => None,
            Self::Unrecorded(_) | Self::Taskless => Some(Listed::new(unresolved).then(self)),
        }
    }
}

impl fmt::Display for WaitOnCi {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recorded => Ok(()),
            Self::Unrecorded(task) => {
                write!(formatter, "; or run `meta wait {task} --on ci` to stop with the wait on record")
            }
            Self::Taskless => formatter
                .write_str("; or claim a task (`meta claim <task> --worktree`), then run `meta wait <task> --on ci`"),
        }
    }
}
