use chrono::TimeDelta;
use meta_log::MemoryLog;

use super::{Beat, Scene};
use meta_golden::Golden;

use crate::request::heartbeat::{HeartbeatRecord, Snapshot, Tick};

const NOTHING: &str = "";
const TORN: &str = "{torn";
const RECORD: &str = "meta-local/heartbeat.json";
const EARLY: TimeDelta = TimeDelta::seconds(0);
const MIDDLE: TimeDelta = TimeDelta::seconds(300);
const LATE: TimeDelta = TimeDelta::seconds(600);
const TICKS: [(Scene, TimeDelta); 3] = [(Scene::Offline, EARLY), (Scene::Orphaned, MIDDLE), (Scene::Orphaned, LATE)];

#[test]
fn a_tick_prints_only_new_flags_and_exits_non_zero_only_for_flags_needing_judgement() {
    let git_dir = tempfile::tempdir().unwrap();
    let log = MemoryLog::default();
    let ticks = TICKS.map(|(scene, at)| Tick::answer(git_dir.path(), &log, scene.answering(), Beat::pulse(at)));
    Golden::for_test(module_path!(), "ticks").debugged(&ticks.map(Result::unwrap)).unwrap();
}

#[test]
fn a_tick_keeps_the_previous_snapshot_beside_the_latest() {
    let git_dir = tempfile::tempdir().unwrap();
    let log = MemoryLog::default();
    assert!(HeartbeatRecord::read(git_dir.path()).unwrap().is_none());
    Tick::answer(git_dir.path(), &log, Scene::Offline.answering(), Beat::pulse(EARLY)).unwrap();
    Tick::answer(git_dir.path(), &log, Scene::Orphaned.answering(), Beat::pulse(MIDDLE)).unwrap();
    let record = HeartbeatRecord::read(git_dir.path()).unwrap().unwrap();
    let before = record.previous().map(Snapshot::taken);
    assert_eq!([Some(record.latest().taken()), before], [Some(Beat::after(MIDDLE)), Some(Beat::after(EARLY))]);
}

#[test]
fn a_damaged_record_is_replaced_by_the_next_tick() {
    let git_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(git_dir.path().join("meta-local")).unwrap();
    std::fs::write(git_dir.path().join(RECORD), TORN).unwrap();
    assert!(HeartbeatRecord::read(git_dir.path()).is_err());
    let ticked =
        Tick::answer(git_dir.path(), &MemoryLog::default(), Scene::Quiet.answering(), Beat::pulse(EARLY)).unwrap();
    assert_eq!(ticked.to_string(), NOTHING);
    assert!(HeartbeatRecord::read(git_dir.path()).unwrap().is_some());
}
