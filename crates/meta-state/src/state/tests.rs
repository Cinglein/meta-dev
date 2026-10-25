use meta_schema::{Actor, Claim, Event, Id, NewConcern, NewTask, Record, Target, Task, TaskRef};

use super::State;
use crate::refusal::Refusal;

const OWNER: &str = "owner";
const ALICE: &str = "alice";
const BOB: &str = "bob";

struct Events;

impl Events {
    fn actor(name: &str) -> Actor {
        name.parse().unwrap()
    }

    fn task(targets: Vec<Target>) -> NewTask {
        NewTask::new(OWNER.parse().unwrap(), targets)
    }

    fn claim(task: Id<Task>, from: Option<&str>) -> Event {
        Event::TaskClaimed(Claim::new(task, from.map(Self::actor)))
    }

    fn concern() -> NewConcern {
        NewConcern::new(OWNER.parse().unwrap(), None, vec![])
    }
}

#[derive(Clone, Copy)]
enum Step {
    Opened,
    Done,
    Released,
    Reopened,
}

const OPENED: Step = Step::Opened;
const DONE: Step = Step::Done;
const RELEASED: Step = Step::Released;
const REOPENED: Step = Step::Reopened;

struct Lifecycle;

impl Lifecycle {
    fn event(task: &NewTask, step: Step) -> Event {
        let reference = TaskRef::new(task.task());
        match step {
            Step::Opened => Event::TaskOpened(task.clone()),
            Step::Done => Event::TaskDone(reference),
            Step::Released => Event::TaskReleased(reference),
            Step::Reopened => Event::TaskReopened(reference),
        }
    }
}

#[derive(Default)]
struct Script {
    records: Vec<Record>,
}

impl Script {
    fn then(mut self, actor: &str, event: Event) -> Self {
        self.records.push(Record::new(Events::actor(actor), event));
        self
    }
}

#[test]
fn done_addresses_targets_and_reopen_undoes_it() {
    let concern = Events::concern();
    let target = Target::Concern(concern.concern());
    let task = Events::task(vec![target]);
    let done = Script::default()
        .then(OWNER, Event::ConcernIdentified(concern))
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(ALICE, Events::claim(task.task(), None))
        .then(ALICE, Lifecycle::event(&task, DONE));
    assert_eq!(State::fold(&done.records).addressed_by(target), Some(task.task()));
    assert_eq!(State::fold(&done.then(BOB, Lifecycle::event(&task, REOPENED)).records).addressed_by(target), None);
}

#[test]
fn claims_are_taken_over_only_by_naming_the_claimant_and_finished_only_by_them() {
    let task = Events::task(vec![]);
    let script = Script::default()
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(ALICE, Events::claim(task.task(), None))
        .then(BOB, Events::claim(task.task(), None))
        .then(BOB, Lifecycle::event(&task, DONE))
        .then(BOB, Events::claim(task.task(), Some(ALICE)))
        .then(ALICE, Lifecycle::event(&task, RELEASED));
    let refused: Vec<String> = State::fold(&script.records).anomalies().iter().map(ToString::to_string).collect();
    let expected = [
        Refusal::ClaimedBy(Events::actor(ALICE)),
        Refusal::HeldBy(Events::actor(ALICE)),
        Refusal::HeldBy(Events::actor(BOB)),
    ];
    assert!(refused.iter().zip(&expected).all(|(found, refusal)| found.ends_with(&refusal.to_string())));
    assert_eq!(refused.len(), expected.len());
}

#[test]
fn invalid_events_are_kept_as_anomalies_and_vetting_changes_nothing() {
    let task = Events::task(vec![]);
    let ghost = Events::task(vec![]);
    let script = Script::default()
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(OWNER, Lifecycle::event(&ghost, DONE));
    let state = State::fold(&script.records);
    assert_eq!(state.anomalies().len(), script.records.len() - state.tasks().len());
    assert_eq!(
        state.vet(&Record::new(Events::actor(OWNER), Lifecycle::event(&ghost, DONE))),
        Err(Refusal::UnknownTask(ghost.task()))
    );
    assert_eq!(state.tasks().len(), State::fold(&script.records).tasks().len());
}
