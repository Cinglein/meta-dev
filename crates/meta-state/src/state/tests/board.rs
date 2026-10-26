use meta_schema::{Block, Blocker, Event, Id, NewIssue, Task, TaskRef};

use super::{ALICE, Events, Items, Lifecycle, OPENED, OWNER, Script};

#[derive(Clone)]
pub struct Board {
    script: Script,
    task: Id<Task>,
    other: TaskRef,
    issue: NewIssue,
}

impl Board {
    pub fn assigned() -> Self {
        let task = Events::task(vec![]);
        let other = Events::task(vec![]);
        let issue = Items::issue();
        let script = Script::default()
            .then(OWNER, Event::IssueReported(issue.clone()))
            .then(OWNER, Lifecycle::event(&task, OPENED))
            .then(OWNER, Lifecycle::event(&other, OPENED))
            .then(ALICE, Events::claim(task.task(), None));
        Self { script, task: task.task(), other: TaskRef::new(other.task()), issue }
    }

    pub fn then(self, event: Event) -> Self {
        Self { script: self.script.then(ALICE, event), ..self }
    }

    pub fn blocking(&self, on: Blocker) -> Event {
        Event::TaskBlocked(Block::new(self.task, on))
    }

    pub fn resumed(&self) -> Event {
        Event::TaskUnblocked(TaskRef::new(self.task))
    }

    pub fn script(&self) -> &Script {
        &self.script
    }

    pub fn task(&self) -> Id<Task> {
        self.task
    }

    pub fn other(&self) -> TaskRef {
        self.other
    }

    pub fn issue(&self) -> &NewIssue {
        &self.issue
    }
}
