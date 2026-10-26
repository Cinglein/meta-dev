use super::{ALICE, BOB, DONE, Events, Lifecycle, OPENED, OWNER, RELEASED, REOPENED, Script};
use crate::refusal::Refusal;
use crate::state::State;
use crate::task_state::TaskStatus;

const CAROL: &str = "carol";
const DAVE: &str = "dave";

fn assigned(script: &Script) -> Vec<String> {
    let state = State::fold(&script.records);
    state.tasks().iter().flat_map(|task| task.assignments().iter().map(|stamp| stamp.actor().to_string())).collect()
}

#[test]
fn several_actors_hold_one_task_and_a_takeover_swaps_exactly_one() {
    let task = Events::task(vec![]);
    let id = task.task();
    let script = Script::default()
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(ALICE, Events::claim(id, None))
        .then(BOB, Events::claim(id, None))
        .then(ALICE, Events::claim(id, None))
        .then(CAROL, Events::claim(id, Some(ALICE)))
        .then(DAVE, Events::claim(id, Some(ALICE)))
        .then(CAROL, Events::claim(id, Some(BOB)));
    let expected = [
        Refusal::AlreadyAssigned(Events::actor(ALICE)),
        Refusal::NotAssigned(Events::actor(ALICE)),
        Refusal::AlreadyAssigned(Events::actor(CAROL)),
    ];
    let refused = script.refused();
    assert!(refused.iter().zip(&expected).all(|(found, refusal)| found.ends_with(&refusal.to_string())));
    assert_eq!(refused.len(), expected.len());
    assert_eq!(assigned(&script), [BOB, CAROL]);
}

#[test]
fn unassigning_leaves_the_others_and_the_last_one_reopens_the_task() {
    let task = Events::task(vec![]);
    let id = task.task();
    let both = Script::default()
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(ALICE, Events::claim(id, None))
        .then(BOB, Events::claim(id, None));
    let one = both.then(ALICE, Lifecycle::event(&task, RELEASED));
    assert_eq!((assigned(&one), one.status()), (vec![BOB.to_owned()], TaskStatus::Assigned));
    let none = one.then(BOB, Lifecycle::event(&task, RELEASED));
    assert_eq!((assigned(&none), none.status()), (Vec::<String>::new(), TaskStatus::Open));
}

#[test]
fn finishing_ends_every_assignment_and_reopening_starts_clean() {
    let task = Events::task(vec![]);
    let id = task.task();
    let done = Script::default()
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(ALICE, Events::claim(id, None))
        .then(BOB, Events::claim(id, None))
        .then(BOB, Lifecycle::event(&task, DONE));
    assert!(assigned(&done).is_empty() && matches!(done.status(), TaskStatus::Done(_)));
    let reopened = done.then(ALICE, Lifecycle::event(&task, REOPENED));
    assert!(assigned(&reopened).is_empty() && reopened.status() == TaskStatus::Open);
    assert_eq!(reopened.then(ALICE, Events::claim(id, None)).refused().len(), 0);
}
