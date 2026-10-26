use super::{ALICE, BOB, DONE, Events, Lifecycle, OPENED, OWNER, RELEASED, REOPENED, Script};
use crate::refusal::Refusal;
use crate::task_state::TaskStatus;

const CAROL: &str = "carol";
const DAVE: &str = "dave";

#[test]
fn several_actors_hold_one_task_and_a_takeover_swaps_exactly_one() {
    let task = Events::task(vec![]);
    let id = task.task();
    let both = Script::default()
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(ALICE, Events::claim(id, None))
        .then(BOB, Events::claim(id, None));
    assert_eq!(both.vet(ALICE, Events::claim(id, None)), Err(Refusal::AlreadyAssigned(Events::actor(ALICE))));
    let swapped = both.then(CAROL, Events::claim(id, Some(ALICE)));
    assert_eq!(swapped.vet(DAVE, Events::claim(id, Some(ALICE))), Err(Refusal::NotAssigned(Events::actor(ALICE))));
    assert_eq!(swapped.vet(CAROL, Events::claim(id, Some(BOB))), Err(Refusal::AlreadyAssigned(Events::actor(CAROL))));
    assert_eq!(swapped.assigned(), [Events::actor(BOB), Events::actor(CAROL)]);
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
    assert_eq!((one.assigned(), one.status()), (vec![Events::actor(BOB)], TaskStatus::Assigned));
    let none = one.then(BOB, Lifecycle::event(&task, RELEASED));
    assert_eq!((none.assigned(), none.status()), (vec![], TaskStatus::Open));
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
    assert!(done.assigned().is_empty() && matches!(done.status(), TaskStatus::Done(_)));
    let reopened = done.then(ALICE, Lifecycle::event(&task, REOPENED));
    assert!(reopened.assigned().is_empty() && reopened.status() == TaskStatus::Open);
    assert_eq!(reopened.vet(ALICE, Events::claim(id, None)), Ok(()));
}
