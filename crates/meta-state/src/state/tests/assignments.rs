use meta_schema::Event;

use super::{ALICE, BOB, DONE, Events, Lifecycle, OPENED, OWNER, RELEASED, REOPENED, Script};
use crate::refusal::Refusal;
use crate::state::State;
use crate::task_state::TaskStatus;

const CAROL: &str = "carol";
const DAVE: &str = "dave";
const V6: &str = r#"{"kind":"SchemaVersion","version":6}"#;
const V7: &str = r#"{"kind":"SchemaVersion","version":7}"#;

#[test]
fn several_actors_hold_one_task_and_a_takeover_swaps_exactly_one() {
    let task = Events::task(vec![]);
    let id = task.task();
    let both = Script::default()
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(ALICE, Events::claim(id, None))
        .then(BOB, Events::claim(id, None));
    assert_eq!(both.vet(ALICE, Events::claim(id, None)), Ok(()));
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
fn finishing_ends_every_assignment_and_only_a_reopen_after_the_v7_marker_restores_them() {
    let task = Events::task(vec![]);
    let id = task.task();
    let marker = |json: &str| serde_json::from_str::<Event>(json).unwrap();
    let finished = |script: Script| {
        let claimed = script.then(ALICE, Events::claim(id, None)).then(BOB, Events::claim(id, None));
        claimed.then(BOB, Lifecycle::event(&task, DONE))
    };
    let opened = Script::default().then(OWNER, marker(V6)).then(OWNER, Lifecycle::event(&task, OPENED));
    let old = finished(opened);
    assert!(old.assigned().is_empty() && matches!(old.status(), TaskStatus::Done(_)));
    let cleared = old.then(ALICE, Lifecycle::event(&task, REOPENED));
    let kept = finished(cleared.clone()).then(OWNER, marker(V7)).then(ALICE, Lifecycle::event(&task, REOPENED));
    let both = vec![Events::actor(ALICE), Events::actor(BOB)];
    assert_eq!((cleared.assigned(), cleared.status()), (vec![], TaskStatus::Open));
    let reclaimed = kept.clone().then(ALICE, Events::claim(id, None));
    assert!(State::fold(&reclaimed.records).anomalies().is_empty() && reclaimed.assigned() == both);
}
