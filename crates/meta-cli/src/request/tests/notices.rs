use meta_log::LogStore;
use meta_schema::Record;

use super::{ALICE, CLAIM, DONE, Meta, OPEN_DASHED, OWNER};
use crate::request::cli_failure::CliFailure;

const GHOST_TASK: &str = "task-00000000000000ff";
const NEWER_MARK: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"SchemaVersion","version":9}"#;

const HELD: &str = "held by owner, alice";

#[test]
fn a_second_holder_prints_who_holds_the_task_and_a_first_holder_prints_nothing() {
    let meta = Meta::default();
    let task = meta.written(OWNER, &OPEN_DASHED);
    let first = meta.run(OWNER, &[CLAIM, &task]).unwrap();
    let second = meta.run(ALICE, &[CLAIM, &task]).unwrap();
    assert!(!first.contains("held by") && second.contains(HELD) && second.contains(&task));
}

#[test]
fn the_first_write_leaves_one_schema_marker_before_the_entry_and_later_writes_add_none() {
    let meta = Meta::default();
    meta.run(OWNER, &OPEN_DASHED).unwrap();
    meta.run(OWNER, &OPEN_DASHED).unwrap();
    let snapshot = meta.0.read().unwrap();
    let marked: Vec<bool> = snapshot.records().iter().map(|record| record.event().schema_version().is_some()).collect();
    assert_eq!(marked, [true, false, false]);
}

#[test]
fn a_refused_write_leaves_the_log_untouched_and_a_newer_log_refuses_writes() {
    let meta = Meta::default();
    assert!(meta.run(OWNER, &[DONE, GHOST_TASK]).is_err());
    assert_eq!(meta.0.read().unwrap().records().len(), 0);
    let newer: Record = serde_json::from_str(NEWER_MARK).unwrap();
    meta.0.append::<CliFailure>(&mut |_| Ok(newer.clone())).unwrap();
    assert!(meta.run(OWNER, &OPEN_DASHED).unwrap_err().to_string().contains("newer"));
}
