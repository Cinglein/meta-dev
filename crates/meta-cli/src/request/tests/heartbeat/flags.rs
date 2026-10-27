use chrono::TimeDelta;
use meta_log::MemoryLog;

use super::{Beat, SETTINGS, Scene};
use crate::request::heartbeat::{LatestFlags, Tick};
use crate::request::output::Output;

const NEVER: &str = "flags:\nheartbeat stale: never ticked\n";
const FRESH: &str = "heartbeat 2026-10-26 12:00Z, 60s ago\nflags:\norphan #16: open, no task\n";
const STALE: &str = "heartbeat stale: 901s since the last tick, limit 900s\norphan #16";
const CLEAN: &str = "heartbeat 2026-10-26 12:00Z, 1s ago\nflags: none\n";
const SKEWED: &str = "heartbeat 2026-10-26 12:00Z, 0s ago\nflags: none\n";
const ZERO: TimeDelta = TimeDelta::seconds(0);
const MINUTE: TimeDelta = TimeDelta::seconds(60);
const LIMIT: TimeDelta = TimeDelta::seconds(900);
const OVER: TimeDelta = TimeDelta::seconds(901);
const SECOND: TimeDelta = TimeDelta::seconds(1);
const BEFORE: TimeDelta = TimeDelta::seconds(-5);

#[test]
fn a_missing_snapshot_is_itself_a_flag() {
    let git_dir = tempfile::tempdir().unwrap();
    let never = LatestFlags::answer(git_dir.path(), &Beat::settings(SETTINGS).unwrap(), Beat::after(ZERO)).unwrap();
    assert!(matches!(&never, Output::Flagged(_)));
    assert_eq!(never.to_string(), NEVER);
}

#[test]
fn flags_show_the_latest_snapshot_with_its_age_until_the_stale_limit_is_passed() {
    let git_dir = tempfile::tempdir().unwrap();
    let limit = Beat::settings(SETTINGS).unwrap();
    Tick::answer(git_dir.path(), &MemoryLog::default(), Scene::Orphaned.answering(), Beat::pulse(ZERO)).unwrap();
    let [fresh, edge, stale] =
        [MINUTE, LIMIT, OVER].map(|age| LatestFlags::answer(git_dir.path(), &limit, Beat::after(age)).unwrap());
    assert!(matches!(&fresh, Output::Listed(_)) && matches!(&edge, Output::Listed(_)));
    assert!(matches!(&stale, Output::Flagged(_)) && fresh.to_string() == FRESH);
    assert!(stale.to_string().contains(STALE));
}

#[test]
fn a_clean_snapshot_reads_none_and_a_future_snapshot_reads_zero_seconds_old() {
    let git_dir = tempfile::tempdir().unwrap();
    let limit = Beat::settings(SETTINGS).unwrap();
    Tick::answer(git_dir.path(), &MemoryLog::default(), Scene::Quiet.answering(), Beat::pulse(ZERO)).unwrap();
    assert_eq!(LatestFlags::answer(git_dir.path(), &limit, Beat::after(SECOND)).unwrap().to_string(), CLEAN);
    assert_eq!(LatestFlags::answer(git_dir.path(), &limit, Beat::after(BEFORE)).unwrap().to_string(), SKEWED);
}
