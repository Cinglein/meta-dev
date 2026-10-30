use meta_schema::Event;

use super::{ALICE, Events, Lifecycle, OPENED, OWNER, Script};
use crate::refusal::Refusal;
use crate::state::State;

const NEWER: &str = r#"{"kind":"SchemaVersion","version":10}"#;
const OLDER: &str = r#"{"kind":"SchemaVersion","version":1}"#;
const PURPOSED: &str =
    r#"{"kind":"TaskPurposed","task":"task-0000000000000009","purpose":{"serves":"project-0000000000000001"}}"#;
const GHOST: &str = "task-0000000000000009";

#[test]
fn a_purpose_for_an_unknown_task_is_refused() {
    let purposed = serde_json::from_str::<Event>(PURPOSED).unwrap();
    assert_eq!(Script::default().vet(ALICE, purposed), Err(Refusal::UnknownTask(GHOST.parse().unwrap())));
}

#[test]
fn a_log_marked_by_a_newer_build_refuses_every_write_and_an_older_mark_refuses_nothing() {
    let task = Events::task(vec![]);
    let marked = |json: &str| Script::default().then(OWNER, serde_json::from_str::<Event>(json).unwrap());
    let newer = marked(NEWER);
    let opened = Lifecycle::event(&task, OPENED);
    let expected = serde_json::from_str::<Event>(NEWER).unwrap().schema_version().unwrap();
    assert_eq!(newer.vet(ALICE, opened.clone()), Err(Refusal::NewerSchema(expected)));
    assert_eq!(marked(OLDER).vet(ALICE, opened), Ok(()));
    assert_eq!(State::fold(&newer.records).anomalies().len(), 0);
}
