use meta_schema::{
    Block, Blocker, Entry, EntryNote, Event, ExternalKind, Id, Issue, NewIssue, NewTask, Target, TaskRef,
};

use super::{ALICE, DONE, Events, Items, Lifecycle, OPENED, OWNER, Script};
use crate::refusal::Refusal;
use crate::task_state::{Stalling, TaskStatus};

const REASON: &str = "dropped";

#[derive(Clone)]
pub struct Board {
    pub script: Script,
    pub task: NewTask,
    pub other: NewTask,
    pub issue: NewIssue,
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
        Self { script, task, other, issue }
    }

    pub fn then(self, event: Event) -> Self {
        Self { script: self.script.then(ALICE, event), ..self }
    }

    pub fn blocking(&self, on: Blocker) -> Event {
        Event::TaskBlocked(Block::new(self.task.task(), on))
    }

    pub fn resumed(&self) -> Event {
        Event::TaskUnblocked(TaskRef::new(self.task.task()))
    }
}

#[test]
fn only_discussion_never_stalls_and_every_wait_is_a_typed_status() {
    let board = Board::assigned();
    let issue = board.issue.issue();
    let other = board.other.task();
    assert_eq!(board.script.status().stalling(), Stalling::Possible);
    let steps = [
        (Blocker::Discussion(issue), TaskStatus::Discussion(issue), Stalling::Never),
        (Blocker::Task(other), TaskStatus::WaitingOnTask(other), Stalling::Possible),
        (
            Blocker::External(ExternalKind::Merge),
            TaskStatus::WaitingOnExternal(ExternalKind::Merge),
            Stalling::Possible,
        ),
    ];
    assert!(steps.into_iter().all(|(on, status, stalling)| {
        let next = board.clone().then(board.blocking(on));
        next.script.status() == status && next.script.status().stalling() == stalling
    }));
    let waiting = board.clone().then(board.blocking(Blocker::Discussion(issue)));
    let resumed = waiting.clone().then(waiting.resumed());
    assert_eq!(
        (resumed.script.status(), resumed.script.status().stalling()),
        (TaskStatus::Assigned, Stalling::Possible)
    );
}

#[test]
fn waits_on_nothing_real_or_on_itself_are_refused() {
    let ghost_issue = Id::<Issue>::generate();
    let ghost_task = Events::task(vec![]).task();
    let board = Board::assigned();
    let own = board.task.task();
    let refusals = [
        (Blocker::Discussion(ghost_issue), Refusal::UnknownTarget(Target::Issue(ghost_issue))),
        (Blocker::Task(ghost_task), Refusal::UnknownTask(ghost_task)),
        (Blocker::Task(own), Refusal::WaitsOnItself),
    ];
    assert!(refusals.into_iter().all(|(on, refusal)| board.script.vet(ALICE, board.blocking(on)) == Err(refusal)));
}

#[test]
fn resuming_needs_a_wait_and_finished_or_retired_tasks_never_wait_or_stall() {
    let board = Board::assigned();
    assert_eq!(board.script.vet(ALICE, board.resumed()), Err(Refusal::NotBlocked));
    let finished = board.clone().then(Lifecycle::event(&board.task, DONE));
    let waiting = finished.blocking(Blocker::External(ExternalKind::Review));
    assert_eq!(finished.script.vet(ALICE, waiting), Err(Refusal::AlreadyDone));
    assert!(matches!(finished.script.status(), TaskStatus::Done(_)));
    assert_eq!(finished.script.status().stalling(), Stalling::Never);
    let note = EntryNote::new(Entry::Task(board.task.task()), REASON.parse().unwrap());
    let retired = board.then(Event::EntryRetired(note));
    assert_eq!((retired.script.status(), retired.script.status().stalling()), (TaskStatus::Retired, Stalling::Never));
}
