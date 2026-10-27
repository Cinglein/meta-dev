use meta_schema::{Actor, Block, Blocker, Claim, Event, ExternalKind, Id, NewTask, Record, Task, TaskRef};
use meta_state::State;

use super::WaitOnCi;
use crate::request::ci::Commit;
use crate::request::cli_failure::{CliFailure, Invalid};

const ACTOR: &str = "agent";
const DIVISION: &str = "features-and-maintenance";
const OLD: &str = "6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94";
const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const RESUME: &str = "`meta resume task-";
const REOPEN: &str = "`meta reopen task-";
const BROKEN: &str = "fetch refused";
const TIMED_OUT: &str = "CI timed out";
const CHECKOUT_OLD: &str = "`git checkout --detach 6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94`";
const UNREADABLE: &str = "meta/log unreadable: ";

struct Board {
    task: Id<Task>,
    records: Vec<Record>,
}

impl Board {
    fn claimed() -> Self {
        let opened = NewTask::new(DIVISION.parse().unwrap(), ACTOR.parse().unwrap(), vec![]);
        let task = opened.task();
        let board = Self { task, records: Vec::new() };
        board.then(|_| Event::TaskOpened(opened)).then(|task| Event::TaskClaimed(Claim::new(task, None)))
    }

    fn then(self, event: impl FnOnce(Id<Task>) -> Event) -> Self {
        let actor: Actor = ACTOR.parse().unwrap();
        Self { records: [self.records, vec![Record::new(actor, event(self.task))]].concat(), ..self }
    }

    fn waiting_at(self, head: &str) -> Self {
        let on = Blocker::External(ExternalKind::Ci);
        self.then(|task| Event::TaskBlocked(Block::new(task, on).at_head(head.parse().unwrap())))
    }

    fn judged(&self, head: &str, branch: Option<Id<Task>>) -> WaitOnCi {
        let head = head.parse().unwrap();
        WaitOnCi::in_state(&State::fold(&self.records), Some(&head), Some(&head), branch)
    }
}

#[test]
fn a_wait_covers_only_its_own_head_whatever_the_branch() {
    let waiting = Board::claimed().waiting_at(OLD);
    let ready = Board::claimed();
    assert_eq!([waiting.judged(OLD, None), waiting.judged(HEAD, None)], [WaitOnCi::Recorded, WaitOnCi::Taskless]);
    assert_eq!(ready.judged(HEAD, Some(ready.task)), WaitOnCi::Unrecorded(ready.task));
}

#[test]
fn a_branch_task_waiting_elsewhere_says_resume_and_a_done_one_says_reopen() {
    let waiting = Board::claimed().waiting_at(OLD);
    let done = Board::claimed().then(|task| Event::TaskDone(TaskRef::new(task)));
    let shown =
        [waiting.judged(HEAD, Some(waiting.task)), done.judged(HEAD, Some(done.task))].map(|wait| wait.to_string());
    assert!(shown[0].contains(RESUME) && shown[1].contains(REOPEN));
}

#[test]
fn a_wait_at_head_leaves_another_unresolved_commit_blocked_and_an_unreadable_log_says_so() {
    let state = State::fold(&Board::claimed().waiting_at(HEAD).records);
    let elsewhere = WaitOnCi::in_state(&state, Some(&OLD.parse().unwrap()), Some(&HEAD.parse().unwrap()), None);
    let Ok(old) = OLD.parse::<Commit>();
    let unreadable = WaitOnCi::judging(Err(CliFailure::raised::<Invalid>(BROKEN)), None, None, [(&old, TIMED_OUT)]);
    assert!(elsewhere.to_string().contains(CHECKOUT_OLD));
    assert!(unreadable.unwrap().to_string().starts_with(UNREADABLE));
}
