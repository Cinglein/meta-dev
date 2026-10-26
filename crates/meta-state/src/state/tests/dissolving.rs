use meta_schema::{Block, Blocker, Entry, Event, ExternalKind, NewTask, Target, TaskRef};

use super::board::Board;
use super::{ALICE, BOB, Events, Lifecycle, OPENED, OWNER, Shown};
use crate::task_state::{Stalling, TaskStatus};

const NOTE: &str = "closed";

#[test]
fn a_wait_dissolves_when_its_issue_is_retired_or_addressed_and_the_task_stalls_normally_again() {
    let board = Board::assigned();
    let issue = board.issue().issue();
    let waiting = board.clone().then(board.blocking(Blocker::Discussion(issue)));
    assert_eq!(waiting.script().status(), TaskStatus::Discussion(issue));
    let retired = waiting.clone().then(Events::retiring(Entry::Issue(issue)));
    assert_eq!(retired.script().status(), TaskStatus::Assigned);
    assert_eq!(retired.script().status().stalling(), Stalling::Possible);
    assert!(Shown::first_task(retired.script()).contains(NOTE) && !Shown::first_task(waiting.script()).contains(NOTE));
    let again = board.blocking(Blocker::Discussion(issue));
    assert_eq!(retired.script().vet(ALICE, again), Err(crate::refusal::Refusal::Closed(Entry::Issue(issue))));
}

#[test]
fn a_wait_dissolves_when_the_task_it_waits_on_is_done_or_retired() {
    let board = Board::assigned();
    let other = board.other();
    let waiting = board.clone().then(board.blocking(Blocker::Task(other.task())));
    assert_eq!(waiting.script().status(), TaskStatus::WaitingOnTask(other.task()));
    let finished = waiting.clone().then(Events::claim(other.task(), None)).then(Event::TaskDone(other));
    assert_eq!(finished.script().status(), TaskStatus::Assigned);
    let retired = waiting.then(Events::retiring(Entry::Task(other.task())));
    assert_eq!(retired.script().status(), TaskStatus::Assigned);
    assert!(Shown::first_task(retired.script()).contains(NOTE));
}

#[test]
fn an_issue_addressed_by_a_finished_task_ends_the_discussion() {
    let board = Board::assigned();
    let issue = board.issue().issue();
    let waiting = board.clone().then(board.blocking(Blocker::Discussion(issue)));
    let fix: NewTask = Events::task(vec![Target::Issue(issue)]);
    let fixed = waiting
        .script()
        .clone()
        .then(OWNER, Lifecycle::event(&fix, OPENED))
        .then(BOB, Events::claim(fix.task(), None))
        .then(BOB, Event::TaskDone(TaskRef::new(fix.task())));
    assert_eq!(fixed.status(), TaskStatus::Assigned);
    let ci = Blocker::External(ExternalKind::Ci);
    let again = fixed.clone().then(ALICE, Event::TaskBlocked(Block::new(board.task(), ci)));
    assert!(Shown::first_task(&fixed).contains(NOTE) && !Shown::first_task(&again).contains(NOTE));
}
