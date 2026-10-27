mod fake_machine;

use std::path::Path;

use chrono::{DateTime, Utc};

use super::Monitor;
use super::patrol::Patrol;
use crate::request::local::local_settings::MonitorSettings;
use crate::request::local::{LocalClone, LocalFile};
use crate::request::output::{Listed, Output};
use fake_machine::FakeMachine;

const NOW: &str = "2026-10-27T12:00:00Z";
const STATE: &str = "meta-local/monitor-state.json";
const MARKED_100: &str = r#"{"marked":[100],"watched":[]}"#;
const SCENE: &str = "  400     1   400 2-00:00:00   0:10.00   1.0 /Applications/Warp.app/Contents/MacOS/stable
  500   400   500   10:00:00   0:01.00   0.0 -zsh
  501   500   501   10:00:00 2:00:00.00   5.0 claude
  502   501   502   09:00:00   0:01.00   0.0 node mcp-server
  503   501   503      00:10   0:00.00   0.0 /bin/zsh -c source /h/.claude/shell-snapshots/snapshot-zsh-1.sh && eval 'cargo test'
  504   503   503   05:00:00 0:40:00.00  99.0 cargo test
  100     1   100   18:00:00 4:51:00.00  27.0 bash -c while true; do :; done
  101   100   100      00:01   0:00.00   0.0 sleep 1
  200     1   200   18:00:00 4:00:00.00  27.0 bash -c while true; do :; done
  300     1   300   18:00:00 4:00:00.00  27.0 bash -c while true; do :; done
   60     1    60      00:01   0:00.00   0.0 /tmp/meta monitor tick
";
const AFTER: &str = "  100     1   100   18:00:05 4:51:05.00  27.0 bash -c while true; do :; done
  102     1   100      00:01   0:00.00   0.0 vim notes
";
const CWDS: &str =
    "p500\nn/p\np501\nn/p\np502\nn/p\np503\nn/p-worktrees/t\np504\nn/p-worktrees/t\np100\nn/p-worktrees/t
p101\nn/p-worktrees/t\np200\nn/p\np300\nn/other\np102\nn/other\np60\nn/p\n";
const UPTIME: &str = "12:00  up 26 days, 19:01, 1 user, load averages: 1.00 2.00 3.01\n";
const LOOKED: &str = "cwds 400,500,501,502,503,504,100,101,200,300,60\n";
const STOPPING: &str = "stop = [\"orphaned\", \"cpu-time\", \"wall-time\"]\nspared = []";
const STOPPED: &str =
    "cwds 400,500,501,502,503,504,100,101,200,300,60\n-TERM -100\npause 5s\ncwds 100,102\n-KILL 100\n";
const OVER: &str =
    "  monitor: agent work pid 100 orphaned past its limit, target -100: bash -c while true; do :; done\n";
const UNKNOWN: &str = "  monitor: no agent marker under a root, pid 200 orphaned past its limit, target 200: bash";
const WATCH: &str =
    "watch:\n  pid 504 cargo test: cpu 2400s flat, 1 ticks since 2026-10-27 12:00Z, last 2026-10-27 12:00Z\n";
const PROMOTING: &str = "watch_ticks = 2\nkept_entries = 2";
const TRAIL: &str = "2026-10-27 12:00Z monitor: no agent marker under a root, pid 200 orphaned past its limit, target 200: bash -c while true; do :; done
2026-10-27 12:00Z monitor: watched long enough, pid 504 cargo test: cpu 2400s flat, 2 ticks since 2026-10-27 12:00Z, last 2026-10-27 12:00Z\n";
const GIT: &str = ".git";
const OTHER_PROJECT: &str = "pid 300";

struct Watch;

impl Watch {
    fn patrolled(host: &FakeMachine, git_dir: &Path, overrides: &str) -> Output {
        let now = NOW.parse::<DateTime<Utc>>().unwrap();
        let checkout = [Path::new("/p"), Path::new("/p-worktrees"), Path::new("/h")];
        let settings =
            MonitorSettings::merged(toml::from_str(overrides).unwrap(), checkout[0], checkout[1], checkout[2]);
        let state = LocalFile::new(git_dir.join(STATE));
        state.text().unwrap().is_empty().then(|| state.replaced(MARKED_100).unwrap());
        Monitor::patrolled(git_dir, &Patrol::new(&settings.unwrap(), host), now).unwrap()
    }
}

#[test]
fn by_default_the_monitor_signals_nothing_and_reports_obvious_and_watched_work() {
    let git_dir = tempfile::tempdir().unwrap();
    let host = FakeMachine::new(&[SCENE, AFTER], CWDS, UPTIME);
    let report = Watch::patrolled(&host, git_dir.path(), "").to_string();
    assert_eq!(host.to_string(), LOOKED);
    assert!(report.contains(OVER) && report.contains(UNKNOWN) && report.contains(WATCH));
    assert!(!report.contains(OTHER_PROJECT));
}

#[test]
fn opted_in_stopping_spares_the_owner_shell_claude_its_children_other_projects_and_a_mixed_group() {
    let git_dir = tempfile::tempdir().unwrap();
    let host = FakeMachine::new(&[SCENE, AFTER], CWDS, UPTIME);
    let found = Watch::patrolled(&host, git_dir.path(), STOPPING);
    assert_eq!(host.to_string(), STOPPED);
    assert!(matches!(found, Output::Flagged(_)));
}

#[test]
fn a_watch_item_that_persists_becomes_obvious_and_the_trail_keeps_no_repeats_within_its_cap() {
    let checkout = tempfile::tempdir().unwrap();
    let clone = LocalClone::new(checkout.path().to_path_buf(), checkout.path().join(GIT));
    let git_dir = clone.git_dir().as_ref();
    let ticks =
        [SCENE, SCENE].map(|scene| Watch::patrolled(&FakeMachine::new(&[scene], CWDS, UPTIME), git_dir, PROMOTING));
    let recent = Monitor::recent(&clone, NOW.parse().unwrap()).unwrap();
    assert!(ticks.iter().all(|tick| matches!(tick, Output::Flagged(_))));
    assert_eq!(Output::Listed(Listed::new("")).raised_with(recent).to_string(), TRAIL);
}
