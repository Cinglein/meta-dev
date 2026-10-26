use meta_schema::{Blocker, Entry, EntryNote, Event, ExternalKind};

use super::blocking::Board;
use super::{ALICE, BOB, DONE, Events, Lifecycle, OPENED, OWNER, RELEASED};
use crate::refusal::Refusal;

const REASON: &str = "dropped";
const CAROL: &str = "carol";

impl Board {
    fn retiring(entry: Entry) -> Event {
        Event::EntryRetired(EntryNote::new(entry, REASON.parse().unwrap()))
    }
}

#[test]
fn waits_on_closed_issues_closed_tasks_cycles_and_second_waits_are_refused() {
    let board = Board::assigned();
    let (issue, other) = (board.issue.issue(), board.other.task());
    let on_issue = board.blocking(Blocker::Discussion(issue));
    let on_other = board.blocking(Blocker::Task(other));
    let retired_issue = board.clone().then(Board::retiring(Entry::Issue(issue)));
    assert_eq!(retired_issue.script.vet(ALICE, on_issue.clone()), Err(Refusal::Closed(Entry::Issue(issue))));
    let done_other = board.clone().then(Events::claim(other, None)).then(Lifecycle::event(&board.other, DONE));
    assert_eq!(done_other.script.vet(ALICE, on_other.clone()), Err(Refusal::Closed(Entry::Task(other))));
    let retired_other = board.clone().then(Board::retiring(Entry::Task(other)));
    assert_eq!(retired_other.script.vet(ALICE, on_other.clone()), Err(Refusal::Closed(Entry::Task(other))));
    let waiting = board.clone().then(on_issue.clone());
    assert_eq!(waiting.script.vet(ALICE, on_other), Err(Refusal::AlreadyWaiting));
    let on_ci = board.blocking(Blocker::External(ExternalKind::Ci));
    assert_eq!(waiting.script.vet(ALICE, on_ci), Err(Refusal::AlreadyWaiting));
}

#[test]
fn a_wait_that_closes_a_loop_of_any_length_is_refused() {
    let board = Board::assigned();
    let third = Events::task(vec![]);
    let (first, second) = (board.task.task(), board.other.task());
    let wait = |from, on| Event::TaskBlocked(meta_schema::Block::new(from, Blocker::Task(on)));
    let chain = board
        .script
        .clone()
        .then(OWNER, Lifecycle::event(&third, OPENED))
        .then(OWNER, wait(first, second))
        .then(OWNER, wait(second, third.task()));
    assert_eq!(chain.vet(OWNER, wait(third.task(), first)), Err(Refusal::WaitCycle));
    assert_eq!(chain.vet(OWNER, wait(third.task(), second)), Err(Refusal::WaitCycle));
}

#[test]
fn only_holders_wait_or_resume_and_the_last_holder_cannot_leave_a_wait() {
    let board = Board::assigned();
    let ci = board.blocking(Blocker::External(ExternalKind::Ci));
    assert_eq!(board.script.vet(BOB, ci.clone()), Err(Refusal::NotAssigned(Events::actor(BOB))));
    let waiting = board.clone().then(ci);
    assert_eq!(waiting.script.vet(BOB, waiting.resumed()), Err(Refusal::NotAssigned(Events::actor(BOB))));
    assert_eq!(waiting.script.vet(ALICE, Lifecycle::event(&board.task, RELEASED)), Err(Refusal::WaitWithoutHolder));
    let shared = waiting.script.clone().then(CAROL, Events::claim(board.task.task(), None));
    assert_eq!(shared.vet(ALICE, Lifecycle::event(&board.task, RELEASED)), Ok(()));
    let unheld = board.other.clone();
    let wait_on_unheld =
        Event::TaskBlocked(meta_schema::Block::new(unheld.task(), Blocker::External(ExternalKind::Ci)));
    assert_eq!(board.script.vet(BOB, wait_on_unheld), Ok(()));
}
