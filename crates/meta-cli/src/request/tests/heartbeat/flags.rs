use meta_log::MemoryLog;

use super::{SETTINGS, at, orphaned, quiet, settings};
use crate::request::heartbeat::{LatestFlags, Tick};
use crate::request::output::Output;

const NEVER: &str = "flags:\nheartbeat stale: never ticked\n";
const FRESH: &str = "heartbeat 2026-10-26 12:00Z, 60s ago\nflags:\norphan #16: open, no task\n";
const STALE: &str = "heartbeat stale: 901s since the last tick, limit 900s\norphan #16";
const CLEAN: &str = "heartbeat 2026-10-26 12:00Z, 1s ago\nflags: none\n";
const SKEWED: &str = "heartbeat 2026-10-26 12:00Z, 0s ago\n";

#[test]
fn a_missing_snapshot_is_itself_a_flag() {
    let git_dir = tempfile::tempdir().unwrap();
    let never = LatestFlags::answer(git_dir.path(), &settings(SETTINGS).unwrap(), at(0)).unwrap();
    assert!(matches!(&never, Output::Flagged(_)));
    assert_eq!(never.to_string(), NEVER);
}

#[test]
fn flags_show_the_latest_snapshot_with_its_age_until_the_stale_limit_is_passed() {
    let git_dir = tempfile::tempdir().unwrap();
    let limit = settings(SETTINGS).unwrap();
    Tick::answer(git_dir.path(), &MemoryLog::default(), orphaned, at(0)).unwrap();
    let [fresh, edge, stale] = [60, 900, 901].map(|age| LatestFlags::answer(git_dir.path(), &limit, at(age)).unwrap());
    assert!(matches!(&fresh, Output::Listed(_)) && matches!(&edge, Output::Listed(_)));
    assert!(matches!(&stale, Output::Flagged(_)) && fresh.to_string() == FRESH);
    assert!(stale.to_string().contains(STALE));
}

#[test]
fn a_clean_snapshot_reads_none_and_a_future_snapshot_reads_zero_seconds_old() {
    let git_dir = tempfile::tempdir().unwrap();
    let limit = settings(SETTINGS).unwrap();
    Tick::answer(git_dir.path(), &MemoryLog::default(), quiet, at(0)).unwrap();
    assert_eq!(LatestFlags::answer(git_dir.path(), &limit, at(1)).unwrap().to_string(), CLEAN);
    let skewed = LatestFlags::answer(git_dir.path(), &limit, at(-5)).unwrap().to_string();
    assert!(skewed.starts_with(SKEWED));
}
