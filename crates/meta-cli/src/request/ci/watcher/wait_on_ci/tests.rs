use meta_schema::{Actor, Block, Blocker, Claim, Event, ExternalKind, Id, NewTask, Record, Task, TaskRef};
use meta_state::State;

use super::WaitOnCi;

const ACTOR: &str = "agent";
const DIVISION: &str = "features-and-maintenance";
const OLD: &str = "6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94";
const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const RESUME: &str = "`meta resume task-";
const REOPEN: &str = "`meta reopen task-";

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
        WaitOnCi::in_state(&State::fold(&self.records), Some(&head.parse().unwrap()), branch)
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
