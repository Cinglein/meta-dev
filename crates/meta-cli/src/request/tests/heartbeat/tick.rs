use chrono::TimeDelta;
use meta_log::MemoryLog;

use super::super::Repository;
use super::{Beat, Scene};
use crate::request::heartbeat::{HeartbeatRecord, Snapshot, Tick};
use crate::request::local::LocalFile;
use crate::request::output::Output;

const FIRST: &str = "unknown github: offline\n";
const ORPHAN: &str = "orphan #16: open, no task\n";
const NOTHING: &str = "";
const TORN: &str = "{torn";
const RECORD: &str = "meta-local/heartbeat.json";
const EARLY: TimeDelta = TimeDelta::seconds(0);
const MIDDLE: TimeDelta = TimeDelta::seconds(300);
const LATE: TimeDelta = TimeDelta::seconds(600);

#[test]
fn a_tick_prints_only_new_flags_and_exits_non_zero_only_for_flags_needing_judgement() {
    let git_dir = Repository::new();
    let log = MemoryLog::default();
    let first = Tick::answer(git_dir.store(), &log, Scene::Offline.answering(), Beat::pulse(EARLY)).unwrap();
    let second = Tick::answer(git_dir.store(), &log, Scene::Orphaned.answering(), Beat::pulse(MIDDLE)).unwrap();
    let third = Tick::answer(git_dir.store(), &log, Scene::Orphaned.answering(), Beat::pulse(LATE)).unwrap();
    assert!(matches!(&first, Output::Flagged(_)) && first.to_string() == FIRST);
    assert!(matches!(&second, Output::Listed(_)) && second.to_string() == ORPHAN);
    assert!(matches!(&third, Output::Listed(_)) && third.to_string() == NOTHING);
}

#[test]
fn a_tick_keeps_the_previous_snapshot_beside_the_latest() {
    let git_dir = Repository::new();
    let log = MemoryLog::default();
    assert!(HeartbeatRecord::read(git_dir.store()).unwrap().is_none());
    Tick::answer(git_dir.store(), &log, Scene::Offline.answering(), Beat::pulse(EARLY)).unwrap();
    Tick::answer(git_dir.store(), &log, Scene::Orphaned.answering(), Beat::pulse(MIDDLE)).unwrap();
    let record = HeartbeatRecord::read(git_dir.store()).unwrap().unwrap();
    let before = record.previous().map(Snapshot::taken);
    assert_eq!([Some(record.latest().taken()), before], [Some(Beat::after(MIDDLE)), Some(Beat::after(EARLY))]);
}

#[test]
fn a_damaged_record_is_replaced_by_the_next_tick() {
    let git_dir = Repository::new();
    LocalFile::new(git_dir.store().entry(RECORD)).replaced(TORN).unwrap();
    assert!(HeartbeatRecord::read(git_dir.store()).is_err());
    let ticked =
        Tick::answer(git_dir.store(), &MemoryLog::default(), Scene::Quiet.answering(), Beat::pulse(EARLY)).unwrap();
    assert_eq!(ticked.to_string(), NOTHING);
    assert!(HeartbeatRecord::read(git_dir.store()).unwrap().is_some());
}
