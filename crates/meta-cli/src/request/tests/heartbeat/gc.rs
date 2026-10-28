use std::time::{Duration, Instant};

use chrono::TimeDelta;
use meta_log::MemoryLog;
use meta_run::{EnvPolicy, Run};

use super::{Beat, Scene};
use crate::request::heartbeat::{GcHook, LatestFlags, Tick};
use crate::request::local::Sweep;
use crate::request::project::GcMode;

const FREE: &str = "[heartbeat]\ngc = \"free\"";
const DRY_RUN: &str = "[heartbeat]\ngc = \"dry-run\"";
const UNKNOWN: &str = "[heartbeat]\ngc = \"sometimes\"";
const RECORD: &str = "meta-local/heartbeat.json";
const PENDING: &str = r#""gc":"Pending""#;
const FREED: &str = r#""gc":{"Freed":"freed x\n"}"#;
const DISABLED_FLAG: &str = "heartbeat gc disabled";
const ZERO: TimeDelta = TimeDelta::seconds(0);
const SHELL: &str = "sh";
const SCRIPTS: [&str; 4] = ["echo freed x", "echo freed x", "echo broken >&2; exit 1", "sleep 2"];
const LIMIT: Duration = Duration::from_secs(5);
const GC_DEFAULT_FLOOR: Duration = Duration::from_secs(100);
const LIMITS: [Duration; 4] = [LIMIT, LIMIT, LIMIT, Duration::from_millis(10)];
const REPORTS: [&str; 4] = [
    r#"{"Planned":"freed x\n"}"#,
    r#"{"Freed":"freed x\n"}"#,
    r#"{"Failed":"broken"}"#,
    r#"{"Unfinished":"`sh -c sleep 2` timed out"}"#,
];

#[test]
fn an_unknown_gc_switch_still_reads_so_the_tick_runs_and_only_dry_run_or_free_sweep() {
    let read = [FREE, DRY_RUN, UNKNOWN].map(|text| Beat::settings(text).is_ok());
    let sweeps = ["", DRY_RUN, FREE, UNKNOWN].map(|text| Beat::settings(text).unwrap().gc_mode().sweep());
    assert_eq!(read, [true, true, true]);
    assert_eq!(sweeps, [Some(Sweep::Planned), Some(Sweep::Planned), Some(Sweep::Done), None]);
    let deadline = Beat::settings("").unwrap().gc_limit().deadline();
    assert!(deadline > Instant::now() + GC_DEFAULT_FLOOR);
}

#[test]
fn gc_runs_in_its_own_child_under_its_own_deadline_and_a_timeout_never_reads_as_freed() {
    let modes = [GcMode::DryRun, GcMode::Free, GcMode::Free, GcMode::Free];
    let hooks = SCRIPTS.iter().zip(modes).zip(LIMITS).map(|((script, mode), limit)| {
        let run = Run::new(SHELL, EnvPolicy::Inherit).args(["-c", script]);
        serde_json::to_string(&GcHook::supervised(run, Instant::now() + limit, mode)).unwrap()
    });
    assert_eq!(hooks.collect::<Vec<_>>(), REPORTS);
    let disabled = GcHook::supervised(Run::new(SHELL, EnvPolicy::Inherit), Instant::now() + LIMIT, GcMode::Disabled);
    assert_eq!(disabled, GcHook::Disabled);
}

#[test]
fn the_flags_snapshot_is_written_first_and_gc_reports_into_it_afterwards() {
    let git_dir = tempfile::tempdir().unwrap();
    Tick::answer(git_dir.path(), &MemoryLog::default(), Scene::Quiet.answering(), Beat::pulse(ZERO)).unwrap();
    let pending = std::fs::read_to_string(git_dir.path().join(RECORD)).unwrap();
    Tick::reported(git_dir.path(), serde_json::from_str(r#"{"Freed":"freed x\n"}"#).unwrap()).unwrap();
    let reported = std::fs::read_to_string(git_dir.path().join(RECORD)).unwrap();
    let flagged = LatestFlags::answer(git_dir.path(), &Beat::settings(UNKNOWN).unwrap(), Beat::after(ZERO)).unwrap();
    assert!(pending.contains(PENDING) && reported.contains(FREED) && flagged.to_string().contains(DISABLED_FLAG));
}
