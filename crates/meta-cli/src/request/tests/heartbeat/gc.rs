use chrono::TimeDelta;
use meta_log::MemoryLog;

use super::{Beat, PLANNED, Scene};
use crate::request::heartbeat::Tick;
use crate::request::local::Sweep;
use crate::request::project::GcMode;

const FREE: &str = "[heartbeat]\ngc = \"free\"";
const DRY_RUN: &str = "[heartbeat]\ngc = \"dry-run\"";
const UNKNOWN: &str = "[heartbeat]\ngc = \"sometimes\"";
const RECORD: &str = "meta-local/heartbeat.json";
const RECORDED: &str = r#""gc":{"Planned":"would free /trees/task-00000000000000b1: 1160 bytes; pr #1 merged\n"}"#;
const ZERO: TimeDelta = TimeDelta::seconds(0);

#[test]
fn the_heartbeat_gc_switch_takes_dry_run_or_free_and_nothing_else() {
    let read = [FREE, DRY_RUN, UNKNOWN].map(|text| Beat::settings(text).is_ok());
    assert_eq!(read, [true, true, false]);
}

#[test]
fn gc_stays_a_dry_run_unless_the_owner_switches_it_to_free() {
    let modes = ["", DRY_RUN, FREE].map(|text| Beat::settings(text).unwrap().gc_mode());
    assert_eq!(modes, [GcMode::DryRun, GcMode::DryRun, GcMode::Free]);
    assert_eq!(modes.map(GcMode::sweep), [Sweep::Planned, Sweep::Planned, Sweep::Done]);
}

#[test]
fn the_snapshot_records_what_gc_did_or_would_do() {
    let git_dir = tempfile::tempdir().unwrap();
    Tick::answer(git_dir.path(), &MemoryLog::default(), Scene::Quiet.answering(), Beat::pulse(ZERO)).unwrap();
    let recorded = std::fs::read_to_string(git_dir.path().join(RECORD)).unwrap();
    assert!(recorded.contains(RECORDED) && PLANNED.ends_with('\n'), "{recorded}");
}
