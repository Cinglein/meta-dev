use meta_schema::{
    Block, Blocker, Entry, EntryNote, Event, ExternalKind, Id, Issue, NewIssue, NewTask, Target, TaskRef,
};

use super::{ALICE, DONE, Events, Items, Lifecycle, OPENED, OWNER, Script};
use crate::refusal::Refusal;
use crate::task_state::{Stalling, TaskStatus};

const REASON: &str = "dropped";

#[derive(Clone)]
struct Board {
    script: Script,
    task: NewTask,
    other: NewTask,
    issue: NewIssue,
}

impl Board {
    fn assigned() -> Self {
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

    fn then(self, event: Event) -> Self {
        Self { script: self.script.then(ALICE, event), ..self }
    }

    fn blocking(&self, on: Blocker) -> Event {
        Event::TaskBlocked(Block::new(self.task.task(), on))
    }

    fn resumed(&self) -> Event {
        Event::TaskUnblocked(TaskRef::new(self.task.task()))
    }
}

#[test]
fn discussion_and_waits_are_typed_statuses_and_never_stall() {
    let board = Board::assigned();
    let issue = board.issue.issue();
    let other = board.other.task();
    assert_eq!(board.script.status().stalling(), Stalling::Possible);
    let steps = [
        (Blocker::Discussion(issue), TaskStatus::Discussion(issue)),
        (Blocker::Task(other), TaskStatus::WaitingOnTask(other)),
        (Blocker::External(ExternalKind::Ci), TaskStatus::WaitingOnExternal(ExternalKind::Ci)),
    ];
    let waiting = steps.into_iter().fold(board, |board, (on, expected)| {
        let blocked = board.blocking(on);
        let next = board.then(blocked);
        assert!(next.script.status() == expected && next.script.status().stalling() == Stalling::Never);
        next
    });
    let resume = waiting.resumed();
    let resumed = waiting.then(resume);
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
