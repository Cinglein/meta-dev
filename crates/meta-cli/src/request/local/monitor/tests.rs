mod fake_machine;

use std::path::Path;

use chrono::{DateTime, TimeDelta, Utc};

use super::Monitor;
use super::patrol::Patrol;
use crate::request::local::LocalClone;
use crate::request::local::local_settings::MonitorSettings;
use crate::request::output::{Listed, Output};
use fake_machine::FakeMachine;

const START: &str = "2026-10-27T12:00:00Z";
const ROOTS: &str = "roots = [\"agents\"]";
const RUNAWAY: &str = "  100     1   100 18:00:00 4:51:00.00  27.0 bash -c while true; do :; done
  101   100   100      00:01   0:00.00   0.0 sleep 1
  200     1   200 18:00:00 4:00:00.00  27.0 bash -c while true; do :; done
  300     1   300 18:00:00   0:01.00   0.0 tail -f /agents/clone/log
   60     1    60 18:00:00   0:01.00   0.0 meta monitor tick
";
const STUBBORN: &str = "  100     1   100 18:00:05 4:51:05.00  27.0 bash -c while true; do :; done\n";
const RUNAWAY_CWDS: &str =
    "p100\nfcwd\nn/agents/clone\np101\nn/agents/clone\np200\nn/home/owner\np60\nn/agents/clone\n";
const HIGH_LOAD: &str = "12:00  up 26 days, 19:01, 1 user, load averages: 1.00 2.00 3.01\n";
const STOPPED: &str = "cwds 100,101,200,300,60\n-TERM -100\npause 5s\ncwds 100\n-KILL -100\n";
const RUNAWAY_FOUND: &str =
    "monitor sent -TERM to pid 100 orphaned past its limit, target -100: bash -c while true; do :; done
monitor sent -KILL to pid 100 orphaned past its limit, target -100: bash -c while true; do :; done
monitor left alone, unsure it is agent work: pid 300 orphaned past its limit, target 300: tail -f /agents/clone/log
monitor: load 3.01 cores, limit 3.00
";
const LIMITS: &str = "   10     1    10      10:00   0:00.00   0.0 sh at-orphan-limit
   11     1    11      10:01   0:00.00   0.0 sh past-orphan-limit
   12    50    12   01:00:00 1:00:01.00 401.0 cargo test
   13    50    12   01:00:00   0:00.00   0.0 sh outside
   14    50    14   08:00:00 1:00:00.00   0.0 sh at-limits
   15    50    15   08:00:01   0:00.00   0.0 sh past-wall-limit
   16     1    16 1-00:00:00   0:00.00   0.0 /usr/local/bin/claude --resume
   17    50    17   08:00:01   0:00.00   0.0 sh mixed-group
   18    50    17   08:00:01   0:00.00   0.0 sh outside
   50     1    50 2-00:00:00   0:00.00   0.0 zsh
   60    50    60      00:01   0:00.00   0.0 meta monitor tick
";
const SURVIVORS: &str = "   12    50    12   01:00:06 1:00:06.00 401.0 cargo test
   17    50    17      00:01   0:00.00   0.0 vim notes
";
const LIMITS_CWDS: &str = "p10\nn/agents\np11\nn/agents\np12\nn/agents\np13\nn/home/owner\np14\nn/agents\np15\nn/agents
p16\nn/agents\np17\nn/agents\np18\nn/home/owner\np50\nn/agents\np60\nn/agents\n";
const LOAD_AT_LIMIT: &str = "12:00:00 up 1 day, 2 users, load average: 9.00, 9.00, 3.00\n";
const LIMITED: &str = "cwds 10,11,12,13,14,15,16,17,18,50,60\n-TERM -11 -15 12 17\npause 5s\ncwds 12,17\n-KILL 12\n";
const LIMITS_FOUND: &str = "monitor sent -TERM to pid 11 orphaned past its limit, target -11: sh past-orphan-limit
monitor sent -TERM to pid 12 past its cpu time limit, target 12: cargo test
monitor sent -TERM to pid 15 past its wall time limit, target -15: sh past-wall-limit
monitor sent -TERM to pid 17 past its wall time limit, target 17: sh mixed-group
monitor sent -KILL to pid 12 past its cpu time limit, target 12: cargo test
monitor: agent processes use 4.01 cores, limit 4.00
";
const QUIET_TABLE: &str = "   60     1    60      00:01   0:00.00   0.0 meta monitor tick\n";
const BLIND: &str = "monitor: process table unreadable, nothing stopped\n";
const QUIET: &str = "monitor: nothing to stop or flag\n";
const KEPT: &str = "2026-10-27 12:00Z monitor: process table unreadable, nothing stopped\n";
const GIT: &str = ".git";
const ZERO: TimeDelta = TimeDelta::seconds(0);
const RECENT: TimeDelta = TimeDelta::seconds(3600);
const PAST: TimeDelta = TimeDelta::seconds(3601);

struct Watch;

impl Watch {
    fn after(offset: TimeDelta) -> DateTime<Utc> {
        START.parse::<DateTime<Utc>>().unwrap() + offset
    }

    fn patrolled(host: &FakeMachine, git_dir: &Path) -> Output {
        let settings = MonitorSettings::merged(toml::from_str(ROOTS).unwrap(), Path::new("/")).unwrap();
        Monitor::patrolled(git_dir, &Patrol::new(&settings, host), Self::after(ZERO)).unwrap()
    }
}

#[test]
fn an_orphaned_loop_under_a_root_is_stopped_and_processes_outside_or_unsure_are_not() {
    let git_dir = tempfile::tempdir().unwrap();
    let host = FakeMachine::new(&[RUNAWAY, STUBBORN], RUNAWAY_CWDS, HIGH_LOAD);
    let found = Watch::patrolled(&host, git_dir.path());
    assert_eq!(host.to_string(), STOPPED);
    assert!(matches!(&found, Output::Flagged(_)));
    assert_eq!(found.to_string(), RUNAWAY_FOUND);
}

#[test]
fn each_limit_stops_only_past_it_and_spares_the_monitor_its_ancestors_and_claude() {
    let git_dir = tempfile::tempdir().unwrap();
    let host = FakeMachine::new(&[LIMITS, SURVIVORS], LIMITS_CWDS, LOAD_AT_LIMIT);
    let found = Watch::patrolled(&host, git_dir.path());
    assert_eq!(host.to_string(), LIMITED);
    assert_eq!(found.to_string(), LIMITS_FOUND);
}

#[test]
fn findings_stay_in_the_local_record_for_the_recent_window_and_an_unreadable_table_is_a_flag() {
    let checkout = tempfile::tempdir().unwrap();
    let clone = LocalClone::new(checkout.path().to_path_buf(), checkout.path().join(GIT));
    let blind = Watch::patrolled(&FakeMachine::new(&[], "", HIGH_LOAD), clone.git_dir().as_ref());
    let quiet = Watch::patrolled(&FakeMachine::new(&[QUIET_TABLE], "", LOAD_AT_LIMIT), clone.git_dir().as_ref());
    let [kept, gone] = [RECENT, PAST].map(|offset| {
        Output::Listed(Listed::new("")).raised_with(Monitor::recent(&clone, Watch::after(offset)).unwrap())
    });
    assert!(matches!(
        (&blind, &quiet, &kept, &gone),
        (Output::Flagged(_), Output::Listed(_), Output::Flagged(_), Output::Listed(_))
    ));
    assert_eq!([blind, quiet, kept, gone].map(|output| output.to_string()), [BLIND, QUIET, KEPT, ""]);
}
