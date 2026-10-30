mod open_tasks;

use meta_schema::{Id, Project, Task};
use thiserror::Error;

use open_tasks::OpenTasks;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum Unservable {
    #[error("{project} is accepted, not instrumented; claim an instruments task first: {instruments}")]
    Accepted { project: Id<Project>, instruments: OpenTasks },
    #[error("{0} is delivered; retire this task and open one with --caused-by <cause> --project {0}")]
    Delivered(Id<Project>),
    #[error("{0} is regressed; retire this task and open one with --caused-by <cause> --project {0}")]
    Regressed(Id<Project>),
}

impl Unservable {
    #[must_use]
    pub fn accepted(project: Id<Project>, instruments: Vec<Id<Task>>) -> Self {
        Self::Accepted { project, instruments: OpenTasks::from(instruments) }
    }
}
