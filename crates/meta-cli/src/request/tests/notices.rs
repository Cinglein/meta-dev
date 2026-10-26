use meta_log::LogStore;

use super::{ALICE, CLAIM, Meta, OPEN_DASHED, OWNER};

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
