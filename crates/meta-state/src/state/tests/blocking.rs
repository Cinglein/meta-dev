use meta_schema::{Blocker, Entry, Event, ExternalKind, Id, Issue, Target, TaskRef};

use super::ALICE;
use super::Events;
use super::board::Board;
use crate::refusal::Refusal;
use crate::task_state::{Stalling, TaskStatus};

#[test]
fn only_discussion_never_stalls_and_every_wait_is_a_typed_status() {
    let board = Board::assigned();
    let issue = board.issue().issue();
    let other = board.other().task();
    assert_eq!(board.script().status().stalling(), Stalling::Possible);
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
        next.script().status() == status && next.script().status().stalling() == stalling
    }));
    let waiting = board.clone().then(board.blocking(Blocker::Discussion(issue)));
    let resumed = waiting.clone().then(waiting.resumed());
    assert_eq!(resumed.script().status(), TaskStatus::Assigned);
    let discussed = [waiting.script().status().discussion(), resumed.script().status().discussion()];
    assert_eq!((resumed.script().status().stalling(), discussed), (Stalling::Possible, [Some(issue), None]));
}

#[test]
fn waits_on_nothing_real_or_on_itself_are_refused() {
    let ghost_issue = Id::<Issue>::generate();
    let ghost_task = Events::task(vec![]).task();
    let board = Board::assigned();
    let own = board.task();
    let refusals = [
        (Blocker::Discussion(ghost_issue), Refusal::UnknownTarget(Target::Issue(ghost_issue))),
        (Blocker::Task(ghost_task), Refusal::UnknownTask(ghost_task)),
        (Blocker::Task(own), Refusal::WaitsOnItself),
    ];
    assert!(refusals.into_iter().all(|(on, refusal)| board.script().vet(ALICE, board.blocking(on)) == Err(refusal)));
}

#[test]
fn resuming_needs_a_wait_and_finished_or_retired_tasks_never_wait_or_stall() {
    let board = Board::assigned();
    assert_eq!(board.script().vet(ALICE, board.resumed()), Err(Refusal::NotBlocked));
    let finished = board.clone().then(Event::TaskDone(TaskRef::new(board.task())));
    let waiting = finished.blocking(Blocker::External(ExternalKind::Review));
    assert_eq!(finished.script().vet(ALICE, waiting), Err(Refusal::AlreadyDone));
    assert!(matches!(finished.script().status(), TaskStatus::Done(_)));
    assert_eq!(finished.script().status().stalling(), Stalling::Never);
    let retired = board.clone().then(Events::retiring(Entry::Task(board.task())));
    assert_eq!(retired.script().status(), TaskStatus::Retired);
    assert_eq!(retired.script().status().stalling(), Stalling::Never);
}
