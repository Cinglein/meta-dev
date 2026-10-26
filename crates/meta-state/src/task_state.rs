mod progress;
mod stalling;
mod task_status;

use std::fmt;

use meta_schema::{Entry, Id, NewTask, PullRequest, Target, Task, Text};

use crate::refusal::Refusal;
use crate::stamp::Stamp;
pub use progress::Progress;
pub use stalling::Stalling;
pub use task_status::TaskStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskState {
    task: NewTask,
    opened: Stamp,
    progress: Progress,
    pull_requests: Vec<PullRequest>,
}

impl TaskState {
    pub fn correct(&mut self, text: Text) {
        self.task = self.task.corrected(text);
    }

    pub fn new(task: NewTask, opened: Stamp) -> Self {
        Self { task, opened, progress: Progress::open(), pull_requests: Vec::new() }
    }

    pub fn entry(&self) -> Entry {
        Entry::Task(self.task.task())
    }

    pub fn new_task(&self) -> &NewTask {
        &self.task
    }

    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    pub fn pull_requests(&self) -> &[PullRequest] {
        &self.pull_requests
    }

    pub fn addressing(&self, target: Target) -> Option<Id<Task>> {
        self.progress.done().and(self.task.addresses().contains(&target).then(|| self.task.task()))
    }

    pub fn advance(&mut self, next: impl FnOnce(&Progress) -> Result<Progress, Refusal>) -> Result<(), Refusal> {
        self.progress = next(&self.progress)?;
        Ok(())
    }

    pub fn dissolve(&mut self) {
        self.progress = self.progress.dissolved();
    }

    pub fn linked(&mut self, pull_request: PullRequest) -> Result<(), Refusal> {
        if self.pull_requests.contains(&pull_request) {
            Err(Refusal::AlreadyLinked(pull_request))
        } else {
            self.pull_requests.push(pull_request);
            Ok(())
        }
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[{}] {} — opened by {}", self.progress.status(), self.task, self.opened)?;
        self.progress.assignments().iter().try_for_each(|stamp| write!(formatter, " — assigned {stamp}"))?;
        self.progress.ended().map_or(Ok(()), |ended| write!(formatter, " — wait on {ended} closed"))?;
        self.pull_requests.iter().try_for_each(|pull_request| write!(formatter, " PR {pull_request}"))
    }
}
