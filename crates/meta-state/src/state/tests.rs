mod assignments;
mod blocking;
mod board;
mod dissolving;
mod links;
mod projects;
mod rating;
mod schema;
mod waiting_rules;

use meta_schema::{
    Actor, Claim, Division, Entry, EntryNote, Event, Id, NewConcern, NewIssue, NewTask, Record, Target, Task, TaskRef,
};

use super::State;
use crate::refusal::Refusal;
use crate::task_state::TaskStatus;

const OWNER: &str = "owner";
const ALICE: &str = "alice";
const BOB: &str = "bob";
const REASON: &str = "dropped";
const DIVISION: &str = "features-and-maintenance";

struct Events;

impl Events {
    fn actor(name: &str) -> Actor {
        name.parse().unwrap()
    }

    fn task(targets: Vec<Target>) -> NewTask {
        NewTask::new(Items::division(), OWNER.parse().unwrap(), targets)
    }

    fn claim(task: Id<Task>, from: Option<&str>) -> Event {
        Event::TaskClaimed(Claim::new(task, from.map(Self::actor)))
    }

    fn retiring(entry: Entry) -> Event {
        Event::EntryRetired(EntryNote::new(entry, REASON.parse().unwrap()))
    }
}

struct Items;

impl Items {
    fn division() -> Division {
        DIVISION.parse().unwrap()
    }

    fn concern() -> NewConcern {
        NewConcern::new(Self::division(), OWNER.parse().unwrap(), None, vec![])
    }

    fn issue() -> NewIssue {
        NewIssue::new(Self::division(), OWNER.parse().unwrap())
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

struct Shown;

impl Shown {
    fn first_task(script: &Script) -> String {
        State::fold(&script.records).tasks().first().map(ToString::to_string).unwrap()
    }
}

#[derive(Clone, Default)]
struct Script {
    records: Vec<Record>,
}

impl Script {
    fn then(mut self, actor: &str, event: Event) -> Self {
        self.records.push(Record::new(Events::actor(actor), event));
        self
    }

    fn vet(&self, actor: &str, event: Event) -> Result<(), Refusal> {
        State::fold(&self.records).vet(&Record::new(Events::actor(actor), event))
    }

    fn status(&self) -> TaskStatus {
        let state = State::fold(&self.records);
        state.tasks().first().map(|task| state.status_of(task)).unwrap()
    }

    fn assigned(&self) -> Vec<Actor> {
        let state = State::fold(&self.records);
        state
            .tasks()
            .iter()
            .flat_map(|task| task.progress().assignments().iter().map(|stamp| stamp.actor().clone()))
            .collect()
    }
}

#[test]
fn done_addresses_targets_and_reopen_undoes_it() {
    let concern = Items::concern();
    let target = Target::Concern(concern.concern());
    let task = Events::task(vec![target]);
    let done = Script::default()
        .then(OWNER, Event::ConcernIdentified(concern))
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(ALICE, Events::claim(task.task(), None))
        .then(ALICE, Lifecycle::event(&task, DONE))
        .then(OWNER, Event::IssueReported(Items::issue()));
    let state = State::fold(&done.records);
    assert_eq!(state.addressed_by(target), Some(task.task()));
    let placed = [
        state.concerns().first().and_then(|concern| concern.new_concern().placement().division().cloned()),
        state.issues().first().and_then(|issue| issue.new_issue().placement().division().cloned()),
    ];
    assert_eq!(placed, [Some(Items::division()), Some(Items::division())]);
    let reopened = done.then(BOB, Lifecycle::event(&task, REOPENED));
    assert_eq!(State::fold(&reopened.records).addressed_by(target), None);
}

#[test]
fn only_assignees_finish_or_release_and_every_refusal_names_the_actor() {
    let task = Events::task(vec![]);
    let open = Script::default().then(OWNER, Lifecycle::event(&task, OPENED));
    let held = open.clone().then(ALICE, Events::claim(task.task(), None));
    let refused = Err(Refusal::NotAssigned(Events::actor(BOB)));
    let verdicts = [
        open.vet(BOB, Lifecycle::event(&task, DONE)),
        held.vet(BOB, Lifecycle::event(&task, DONE)),
        held.vet(BOB, Lifecycle::event(&task, RELEASED)),
    ];
    assert_eq!(verdicts, [refused.clone(), refused.clone(), refused]);
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
