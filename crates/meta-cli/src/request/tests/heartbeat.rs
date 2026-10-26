use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use meta_log::MemoryLog;
use meta_pull_query::{LiveQuery, LiveReply};
use meta_status::{Github, GithubFailure};

use super::PROGRAM;
use crate::request::Request;
use crate::request::heartbeat::{Deadline, Heartbeat, HeartbeatRecord, LatestFlags, LaunchdPlist, Tick};
use crate::request::local::LocalClone;
use crate::request::output::Output;
use crate::request::project::Config;

const START: &str = "2026-10-26T12:00:00Z";
const OFFLINE: &str = "offline";
const ORPHAN_REPLY: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[
{"number":16,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}]}}}}"#;
const MAIN: &str = r#""main""#;
const REQUIRED: &str = r#"["ci"]"#;
const SETTINGS: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 60\nstale_seconds = 900";
const OVERLONG_TICK: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 300\nstale_seconds = 900";
const SHORT_STALE: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 60\nstale_seconds = 300";
const ZERO_TICK: &str = "[heartbeat]\ninterval_seconds = 300\ntick_seconds = 0\nstale_seconds = 900";
const UNKNOWN_KEY: &str = "[heartbeat]\ninterval = 300";

fn at(offset: i64) -> DateTime<Utc> {
    START.parse::<DateTime<Utc>>().unwrap() + TimeDelta::seconds(offset)
}

fn offline(_: &LiveQuery) -> Github {
    Github::Unreached(GithubFailure::new(OFFLINE))
}

fn orphaned(_: &LiveQuery) -> Github {
    let rules = [(serde_json::from_str(MAIN).unwrap(), serde_json::from_str(REQUIRED).unwrap())];
    let pulls = serde_json::from_str::<LiveReply>(ORPHAN_REPLY).unwrap().answered().unwrap();
    Github::Reached(pulls.ruled(rules.into_iter().collect()).unwrap())
}

fn settings(text: &str) -> Result<crate::request::project::HeartbeatSettings, crate::request::cli_failure::CliFailure> {
    toml::from_str::<Config>(text).unwrap().heartbeat_settings()
}

#[test]
fn a_tick_prints_only_new_flags_keeps_the_previous_snapshot_and_exits_by_judgement() {
    let git_dir = tempfile::tempdir().unwrap();
    let log = MemoryLog::default();
    let first = Tick::answer(git_dir.path(), &log, offline, at(0)).unwrap();
    assert!(matches!(&first, Output::Flagged(_)) && first.to_string() == "unknown github: offline\n");
    let second = Tick::answer(git_dir.path(), &log, orphaned, at(300)).unwrap();
    assert!(matches!(&second, Output::Listed(_)) && second.to_string() == "orphan #16: open, no task\n");
    let third = Tick::answer(git_dir.path(), &log, orphaned, at(600)).unwrap();
    assert!(matches!(&third, Output::Listed(_)) && third.to_string().is_empty());
    let record = HeartbeatRecord::read(git_dir.path()).unwrap().unwrap();
    assert_eq!([record.latest().at(), record.previous().map(|snapshot| snapshot.at()).unwrap()], [at(600), at(300)]);
}

#[test]
fn flags_show_the_latest_snapshot_with_its_age_and_a_stale_or_missing_one_is_itself_a_flag() {
    let git_dir = tempfile::tempdir().unwrap();
    let limit = settings(SETTINGS).unwrap();
    let never = LatestFlags::answer(git_dir.path(), &limit, at(0)).unwrap();
    assert!(matches!(&never, Output::Flagged(_)) && never.to_string() == "flags:\nheartbeat stale: never ticked\n");
    Tick::answer(git_dir.path(), &MemoryLog::default(), orphaned, at(0)).unwrap();
    let fresh = LatestFlags::answer(git_dir.path(), &limit, at(60)).unwrap();
    assert!(matches!(&fresh, Output::Listed(_)));
    assert_eq!(fresh.to_string(), "heartbeat 2026-10-26 12:00Z, 60s ago\nflags:\norphan #16: open, no task\n");
    let edge = LatestFlags::answer(git_dir.path(), &limit, at(900)).unwrap();
    assert!(matches!(&edge, Output::Listed(_)));
    let stale = LatestFlags::answer(git_dir.path(), &limit, at(901)).unwrap();
    assert!(matches!(&stale, Output::Flagged(_)));
    assert!(stale.to_string().contains("heartbeat stale: 901s since the last tick, limit 900s\norphan #16"));
    let skewed = LatestFlags::answer(git_dir.path(), &limit, at(-5)).unwrap();
    assert!(skewed.to_string().starts_with("heartbeat 2026-10-26 12:00Z, 0s ago\n"));
}

#[test]
fn a_clean_snapshot_reads_flags_none_and_a_damaged_record_is_refused_by_flags_but_replaced_by_a_tick() {
    let git_dir = tempfile::tempdir().unwrap();
    let limit = settings(SETTINGS).unwrap();
    std::fs::create_dir(git_dir.path().join("meta-local")).unwrap();
    std::fs::write(git_dir.path().join("meta-local/heartbeat.json"), "{torn").unwrap();
    assert!(LatestFlags::answer(git_dir.path(), &limit, at(0)).is_err());
    let blank = |_: &LiveQuery| {
        let rules = [(serde_json::from_str(MAIN).unwrap(), serde_json::from_str(REQUIRED).unwrap())];
        let reply = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}}}}"#;
        let pulls = serde_json::from_str::<LiveReply>(reply).unwrap().answered().unwrap();
        Github::Reached(pulls.ruled(rules.into_iter().collect()).unwrap())
    };
    let ticked = Tick::answer(git_dir.path(), &MemoryLog::default(), blank, at(0)).unwrap();
    assert!(ticked.to_string().is_empty());
    let shown = LatestFlags::answer(git_dir.path(), &limit, at(1)).unwrap();
    assert_eq!(shown.to_string(), "heartbeat 2026-10-26 12:00Z, 1s ago\nflags: none\n");
}

#[test]
fn settings_need_a_tick_shorter_than_the_interval_which_is_shorter_than_the_stale_limit() {
    let valid = settings(SETTINGS).unwrap();
    assert_eq!([valid.interval().to_string(), valid.tick_limit().as_secs().to_string()], ["300", "60"]);
    assert_eq!(valid.stale_limit(), Duration::from_secs(900));
    let refused = [OVERLONG_TICK, SHORT_STALE, ZERO_TICK, UNKNOWN_KEY].map(|text| settings(text).is_err());
    assert_eq!(refused, [true; 4]);
    assert!(settings("").is_ok());
}

#[test]
fn a_deadline_returns_finished_work_and_gives_up_on_work_that_never_finishes() {
    assert_eq!(Deadline::within(Duration::from_secs(5), || 7), Some(7));
    let (_held, blocked) = std::sync::mpsc::channel::<()>();
    assert_eq!(Deadline::within(Duration::from_millis(1), move || blocked.recv().is_ok()), None);
}

#[test]
fn heartbeat_names_parse_and_anything_else_is_refused() {
    let parse = |word: &str| Request::try_from([PROGRAM, "heartbeat", word].map(String::from).to_vec());
    assert!(matches!(parse("tick"), Ok(Request::Heartbeat(Heartbeat::Tick))));
    assert!(matches!(parse("flags"), Ok(Request::Heartbeat(Heartbeat::Flags))));
    assert!(matches!(parse("plist"), Ok(Request::Heartbeat(Heartbeat::Plist))));
    assert!(parse("install").is_err());
    assert!(Request::try_from([PROGRAM, "heartbeat"].map(String::from).to_vec()).is_err());
}

#[test]
fn the_plist_runs_tick_in_the_clone_on_the_policy_interval_and_escapes_markup() {
    let clone = LocalClone::new(PathBuf::from("/work/a&b"), PathBuf::from("/work/a&b/.git"));
    let plist = LaunchdPlist::new(PathBuf::from("/bin/meta"), &clone, &settings(SETTINGS).unwrap(), OsString::from("/usr/bin"));
    let text = plist.to_string();
    let expected = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/request/tests/heartbeat.plist")).unwrap();
    assert_eq!(text, expected);
}
