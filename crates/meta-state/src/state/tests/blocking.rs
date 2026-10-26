use meta_schema::{Block, Blocker, Entry, EntryNote, Event, ExternalKind, Id, Issue, NewIssue, NewTask, Target, TaskRef};

use super::{ALICE, DONE, Events, Lifecycle, OPENED, OWNER, Script};
use crate::refusal::Refusal;
use crate::task_state::TaskStatus;

const REASON: &str = "dropped";

struct Board {
    script: Script,
    task: NewTask,
    other: NewTask,
    issue: NewIssue,
}

impl Board {
    fn assigned() -> Self {
        let (task, other, issue) = (Events::task(vec![]), Events::task(vec![]), Events::issue());
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

    fn blocked_on(self, on: Blocker) -> Self {
        let task = self.task.task();
        self.then(Event::TaskBlocked(Block::new(task, on)))
    }

    fn resumed(self) -> Self {
        let task = TaskRef::new(self.task.task());
        self.then(Event::TaskUnblocked(task))
    }

    fn status(&self) -> TaskStatus {
        self.script.status()
    }

    fn last_refusal(&self) -> Option<String> {
        self.script.refused().pop()
    }

    fn refused_with(&self, refusal: &Refusal) -> bool {
        self.last_refusal().is_some_and(|found| found.ends_with(&refusal.to_string()))
    }
}

#[test]
fn discussion_and_waits_are_typed_statuses_and_never_stall() {
    let board = Board::assigned();
    let (issue, other) = (board.issue.issue(), board.other.task());
    assert!(board.status().may_stall());
    let steps = [
        (Blocker::Discussion(issue), TaskStatus::Discussion(issue)),
        (Blocker::Task(other), TaskStatus::WaitingOnTask(other)),
        (Blocker::External(ExternalKind::Ci), TaskStatus::WaitingOnExternal(ExternalKind::Ci)),
    ];
    let board = steps.into_iter().fold(board, |board, (on, expected)| {
        let board = board.blocked_on(on);
        assert!(board.status() == expected && !board.status().may_stall());
        board
    });
    let resumed = board.resumed();
    assert!(resumed.status() == TaskStatus::Assigned && resumed.status().may_stall());
}

#[test]
fn waits_on_nothing_real_or_on_itself_are_refused() {
    let ghost_issue = Id::<Issue>::generate();
    let ghost_task = Events::task(vec![]).task();
    let own = Board::assigned().task.task();
    let refusals = [
        (Blocker::Discussion(ghost_issue), Refusal::UnknownTarget(Target::Issue(ghost_issue))),
        (Blocker::Task(ghost_task), Refusal::UnknownTask(ghost_task)),
        (Blocker::Task(own), Refusal::WaitsOnItself),
    ];
    refusals.into_iter().for_each(|(on, refusal)| {
        let refused = Board::assigned().blocked_on(on);
        assert!(refused.refused_with(&refusal) && refused.status() == TaskStatus::Assigned);
    });
}

#[test]
fn resuming_needs_a_wait_and_finished_or_retired_tasks_never_wait_or_stall() {
    let not_blocked = Board::assigned().resumed();
    assert!(not_blocked.refused_with(&Refusal::NotBlocked));
    let task = not_blocked.task.clone();
    let finished = not_blocked.then(Lifecycle::event(&task, DONE));
    let waiting = finished.blocked_on(Blocker::External(ExternalKind::Review));
    assert!(waiting.refused_with(&Refusal::AlreadyDone));
    assert!(matches!(waiting.status(), TaskStatus::Done(_)) && !waiting.status().may_stall());
    let note = EntryNote::new(Entry::Task(task.task()), REASON.parse().unwrap());
    let retired = Board::assigned().then(Event::EntryRetired(note));
    assert!(retired.status() == TaskStatus::Retired && !retired.status().may_stall());
    let _ = (OWNER, ALICE);
}
