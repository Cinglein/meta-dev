mod task_status;

use std::fmt;

use meta_schema::{Entry, Id, NewTask, Target, Task, Text};

use crate::refusal::Refusal;
use crate::stamp::Stamp;
use task_status::TaskStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskState {
    task: NewTask,
    opened: Stamp,
    status: TaskStatus,
}

impl TaskState {
    pub fn correct(&mut self, text: Text) {
        self.task = self.task.corrected(text);
    }

    pub fn new(task: NewTask, opened: Stamp) -> Self {
        Self { task, opened, status: TaskStatus::Open }
    }

    pub fn entry(&self) -> Entry {
        Entry::Task(self.task.task())
    }

    pub fn new_task(&self) -> &NewTask {
        &self.task
    }

    pub fn status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn addressing(&self, target: Target) -> Option<Id<Task>> {
        self.status.done().and(self.task.addresses().contains(&target).then(|| self.task.task()))
    }

    pub fn advance(&mut self, next: Result<TaskStatus, Refusal>) -> Result<(), Refusal> {
        self.status = next?;
        Ok(())
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[{}] {} — opened by {}", self.status, self.task, self.opened)
    }
}
