use meta_log::MemoryLog;

use super::{at, offline, orphaned, quiet};
use crate::request::heartbeat::{HeartbeatRecord, Tick};
use crate::request::output::Output;

const FIRST: &str = "unknown github: offline\n";
const ORPHAN: &str = "orphan #16: open, no task\n";
const TORN: &str = "{torn";
const RECORD: &str = "meta-local/heartbeat.json";

#[test]
fn a_tick_prints_only_new_flags_and_exits_non_zero_only_for_flags_needing_judgement() {
    let git_dir = tempfile::tempdir().unwrap();
    let log = MemoryLog::default();
    let first = Tick::answer(git_dir.path(), &log, offline, at(0)).unwrap();
    let second = Tick::answer(git_dir.path(), &log, orphaned, at(300)).unwrap();
    let third = Tick::answer(git_dir.path(), &log, orphaned, at(600)).unwrap();
    assert!(matches!(&first, Output::Flagged(_)) && first.to_string() == FIRST);
    assert!(matches!(&second, Output::Listed(_)) && second.to_string() == ORPHAN);
    assert!(matches!(&third, Output::Listed(_)) && third.to_string().is_empty());
}

#[test]
fn a_tick_keeps_the_previous_snapshot_beside_the_latest() {
    let git_dir = tempfile::tempdir().unwrap();
    let log = MemoryLog::default();
    assert!(HeartbeatRecord::read(git_dir.path()).unwrap().is_none());
    Tick::answer(git_dir.path(), &log, offline, at(0)).unwrap();
    Tick::answer(git_dir.path(), &log, orphaned, at(300)).unwrap();
    let record = HeartbeatRecord::read(git_dir.path()).unwrap().unwrap();
    let before = record.previous().map(crate::request::heartbeat::Snapshot::at);
    assert_eq!([Some(record.latest().at()), before], [Some(at(300)), Some(at(0))]);
}

#[test]
fn a_damaged_record_is_replaced_by_the_next_tick() {
    let git_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(git_dir.path().join("meta-local")).unwrap();
    std::fs::write(git_dir.path().join(RECORD), TORN).unwrap();
    assert!(HeartbeatRecord::read(git_dir.path()).is_err());
    let ticked = Tick::answer(git_dir.path(), &MemoryLog::default(), quiet, at(0)).unwrap();
    assert_eq!(ticked.to_string(), "");
    assert!(HeartbeatRecord::read(git_dir.path()).unwrap().is_some());
}
