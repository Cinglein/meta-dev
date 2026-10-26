use meta_schema::{BlockerVisitor, ExternalKind, Id, Issue, Task};

use crate::task_state::task_status::TaskStatus;

#[derive(Clone, Copy, Debug)]
pub struct Waiting;

impl BlockerVisitor for Waiting {
    type Output = TaskStatus;

    fn discussion(self, issue: Id<Issue>) -> TaskStatus {
        TaskStatus::Discussion(issue)
    }

    fn task(self, task: Id<Task>) -> TaskStatus {
        TaskStatus::WaitingOnTask(task)
    }

    fn external(self, kind: ExternalKind) -> TaskStatus {
        TaskStatus::WaitingOnExternal(kind)
    }
}
