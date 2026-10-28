use meta_schema::{Block, Blocker, Entry, Event, ExternalKind, TaskRef};

use super::board::Board;
use super::{ALICE, BOB, Events, Lifecycle, OPENED, OWNER, State};
use crate::refusal::Refusal;

const CAROL: &str = "carol";

#[test]
fn waits_on_closed_issues_closed_tasks_and_second_waits_are_refused() {
    let board = Board::assigned();
    let issue = board.issue().issue();
    let other = board.other().task();
    let on_issue = board.blocking(Blocker::Discussion(issue));
    let on_other = board.blocking(Blocker::Task(other));
    let retired_issue = board.clone().then(Events::retiring(Entry::Issue(issue)));
    assert_eq!(retired_issue.script().vet(ALICE, on_issue.clone()), Err(Refusal::Closed(Entry::Issue(issue))));
    let done_other = board.clone().then(Events::claim(other, None)).then(Event::TaskDone(board.other()));
    assert_eq!(done_other.script().vet(ALICE, on_other.clone()), Err(Refusal::Closed(Entry::Task(other))));
    let retired_other = board.clone().then(Events::retiring(Entry::Task(other)));
    assert_eq!(retired_other.script().vet(ALICE, on_other.clone()), Err(Refusal::Closed(Entry::Task(other))));
    let waiting = board.clone().then(on_issue);
    assert_eq!(waiting.script().vet(ALICE, on_other), Err(Refusal::AlreadyWaiting));
    let on_ci = board.blocking(Blocker::External(ExternalKind::Ci));
    assert_eq!(waiting.script().vet(ALICE, on_ci), Err(Refusal::AlreadyWaiting));
}

#[test]
fn a_wait_that_closes_a_loop_of_any_length_is_refused() {
    let board = Board::assigned();
    let third = Events::task(vec![]);
    let first = board.task();
    let second = board.other().task();
    let wait = |from, on| Event::TaskBlocked(Block::new(from, Blocker::Task(on)));
    let chain = board
        .script()
        .clone()
        .then(OWNER, Lifecycle::event(&third, OPENED))
        .then(OWNER, Events::claim(second, None))
        .then(OWNER, Events::claim(third.task(), None))
        .then(ALICE, wait(first, second))
        .then(OWNER, wait(second, third.task()));
    assert_eq!(chain.vet(OWNER, wait(third.task(), first)), Err(Refusal::WaitCycle));
    assert_eq!(chain.vet(OWNER, wait(third.task(), second)), Err(Refusal::WaitCycle));
}

#[test]
fn only_holders_wait_or_resume_and_the_last_holder_cannot_leave_a_wait() {
    let board = Board::assigned();
    let ci = board.blocking(Blocker::External(ExternalKind::Ci));
    assert_eq!(board.script().vet(BOB, ci.clone()), Err(Refusal::NotAssigned(Events::actor(BOB))));
    let waiting = board.clone().then(ci);
    assert_eq!(waiting.script().vet(BOB, waiting.resumed()), Err(Refusal::NotAssigned(Events::actor(BOB))));
    let leaving = Event::TaskReleased(TaskRef::new(board.task()));
    assert_eq!(waiting.script().vet(ALICE, leaving.clone()), Err(Refusal::WaitWithoutHolder));
    let shared = waiting.script().clone().then(CAROL, Events::claim(board.task(), None));
    assert_eq!(shared.vet(ALICE, leaving), Ok(()));
    let waited = State::fold(&shared.records);
    assert_eq!(waited.tasks().first().and_then(|task| task.progress().waited_by()), Some(&Events::actor(ALICE)));
    let unheld = Block::new(board.other().task(), Blocker::External(ExternalKind::Ci));
    assert_eq!(board.script().vet(BOB, Event::TaskBlocked(unheld)), Err(Refusal::NotAssigned(Events::actor(BOB))));
}
