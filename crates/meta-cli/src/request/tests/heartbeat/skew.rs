use chrono::TimeDelta;
use meta_log::MemoryLog;

use super::{Beat, SETTINGS, Scene};
use crate::request::heartbeat::{LatestFlags, Tick};
use crate::request::output::Output;

const TOLERATED: &str = "heartbeat 2026-10-26 12:00Z, 0s ago\nflags: none\n";
const SKEWED: &str = "heartbeat clock skew: snapshot is 3600s in the future, limit 60s\n";
const ZERO: TimeDelta = TimeDelta::seconds(0);
const SLIGHTLY_BEFORE: TimeDelta = TimeDelta::seconds(-60);
const FAR_BEFORE: TimeDelta = TimeDelta::seconds(-3600);

#[test]
fn a_snapshot_a_little_in_the_future_is_tolerated() {
    let git_dir = tempfile::tempdir().unwrap();
    let limit = Beat::settings(SETTINGS).unwrap();
    Tick::answer(git_dir.path(), &MemoryLog::default(), Scene::Quiet.answering(), Beat::after(ZERO)).unwrap();
    let shown = LatestFlags::answer(git_dir.path(), &limit, Beat::after(SLIGHTLY_BEFORE)).unwrap();
    assert!(matches!(&shown, Output::Listed(_)) && shown.to_string() == TOLERATED);
}

#[test]
fn a_snapshot_far_in_the_future_is_a_clock_skew_flag() {
    let git_dir = tempfile::tempdir().unwrap();
    let limit = Beat::settings(SETTINGS).unwrap();
    Tick::answer(git_dir.path(), &MemoryLog::default(), Scene::Quiet.answering(), Beat::after(ZERO)).unwrap();
    let shown = LatestFlags::answer(git_dir.path(), &limit, Beat::after(FAR_BEFORE)).unwrap();
    assert!(matches!(&shown, Output::Flagged(_)) && shown.to_string().contains(SKEWED));
}
