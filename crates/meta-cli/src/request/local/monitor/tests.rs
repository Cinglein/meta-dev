mod fake_machine;
mod stale_marks;

use std::path::Path;

use chrono::{DateTime, TimeDelta, Utc};
use meta_golden::Golden;

use super::Monitor;
use super::patrol::Patrol;
use super::silence::Silence;
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::local::local_settings::MonitorSettings;
use crate::request::local::{LocalClone, LocalFile};
use crate::request::output::{Listed, Output};
use crate::request::project::{CiSettings, Config, HeartbeatSettings};
use fake_machine::FakeMachine;

const NOW: &str = "2026-10-27T12:00:00Z";
const STATE: &str = "meta-local/monitor-state.json";
const MARKED: &str = r#"{"marked":{"100":{"started":"2026-10-26T18:00:00Z","seen":"2026-10-27T12:00:00Z"},
"110":{"started":"2026-10-27T11:50:00Z","seen":"2026-10-27T12:00:00Z"}},"watched":[]}"#;
const SCENE: &str = "  400     1   400 2-00:00:00   0:10.00   1.0 /Applications/Warp.app/Contents/MacOS/stable
  500   400   500   10:00:00   0:01.00   0.0 -zsh
  501   500   501   10:00:00 2:00:00.00   5.0 /usr/local/bin/claude
  502   501   502   09:00:00   0:01.00   0.0 node mcp-server
  503   501   503      00:10   0:00.00   0.0 /bin/zsh -c source /h/.claude/shell-snapshots/snapshot-zsh-1.sh && eval 'cargo test'
  504   503   503   05:00:00 0:40:00.00  99.0 cargo test
  100     1   100   18:00:00 4:51:00.00  27.0 bash -c while true; do :; done
  101   100   100      00:01   0:00.00   0.0 sleep 1
  106     1   106   05:00:00 0:40:00.00  50.0 cargo test
  110     1   110      10:00   0:00.00   0.0 sleep 600
  120     1   120   18:00:00   0:01.00   0.0 tail -f /p/log
  130     1   130      09:00 0:40:00.00  99.0 cargo run
  140     1   140   18:00:00   0:01.00   0.0 /bin/bash -c source env.sh && make
  200     1   200   18:00:00 4:00:00.00  27.0 bash -c while true; do :; done
  300     1   300   18:00:00 4:00:00.00  27.0 bash -c while true; do :; done
   60     1    60      00:01   0:00.00   0.0 /tmp/meta monitor tick
";
const AFTER: &str = "  100     1   100   18:00:05 4:51:05.00  27.0 bash -c while true; do :; done
  102     1   100      00:01   0:00.00   0.0 vim notes
  105     1   100      00:01   0:00.00   0.0 bash -c while true; do :; done
";
const CWDS: &str =
    "p500\nn/p\np501\nn/p\np502\nn/p\np503\nn/p-worktrees/t\np504\nn/p-worktrees/t\np505\nn/p-worktrees/t
p100\nn/p-worktrees/t\np101\nn/p-worktrees/t\np105\nn/p-worktrees/t\np106\nn/other\np110\nn/p-worktrees/t\np120\nn/other
p130\nn/p\np140\nn/p\np200\nn/p\np201\nn/p\np300\nn/other\np102\nn/other\np60\nn/p\n";
const UPTIME: &str = "12:00  up 26 days, 19:01, 1 user, load averages: 1.00 2.00 3.01\n";
const STOPPING: &str = "stop = [\"orphaned\", \"cpu-time\", \"wall-time\", \"silent\"]\nspared = []";
const SILENT: &str = r#"{"task":"task-00000000000000c0","worktree":"/p-worktrees/t",
"quiet":{"secs":4000,"nanos":0},"limit":{"secs":1800,"nanos":0}}"#;
const PROMOTING: &str = "watch_ticks = 2\nkept_entries = 6";
const LOOP: [&str; 2] = ["  100     1   100", "  201     1   201"];
const JOINED: [&str; 2] =
    ["  101   100   100", "  505   503   503   05:00:00 0:40:00.00  99.0 cargo test\n  101   100   100"];
const RISING: [&str; 2] = ["0:40:00.00  99.0", "0:45:00.00  99.0"];
const LATER: [TimeDelta; 5] =
    [TimeDelta::zero(), TimeDelta::minutes(5), TimeDelta::minutes(10), TimeDelta::minutes(61), TimeDelta::minutes(62)];
const BROKEN: &str = "settings unreadable";
const GIT: &str = ".git";

struct Watch;

impl Watch {
    fn patrolled(host: &FakeMachine, git_dir: &Path, overrides: &str, offset: TimeDelta) -> Output {
        let now = NOW.parse::<DateTime<Utc>>().unwrap() + offset;
        let checkout = [Path::new("/p"), Path::new("/p-worktrees"), Path::new("/h")];
        let settings =
            MonitorSettings::merged(toml::from_str(overrides).unwrap(), checkout[0], &[checkout[1]], checkout[2]);
        let state = LocalFile::new(git_dir.join(STATE));
        state.text().unwrap().is_empty().then(|| state.replaced(MARKED).unwrap());
        let silences = [serde_json::from_str::<Silence>(SILENT).unwrap()];
        Monitor::patrolled(git_dir, &Patrol::new(&settings.unwrap(), host, &silences), now).unwrap()
    }

    fn shown(host: &FakeMachine, after: &str) -> String {
        format!("{host}{after}\n")
    }

    fn reported(overrides: &str) -> String {
        let git_dir = tempfile::tempdir().unwrap();
        let host = FakeMachine::new(&[SCENE, AFTER], CWDS, UPTIME);
        let report = Watch::patrolled(&host, git_dir.path(), overrides, TimeDelta::zero());
        Watch::shown(&host, &format!("succeeds: {}\n{report}", report.exit_code() == std::process::ExitCode::SUCCESS))
    }
}

#[test]
fn by_default_the_monitor_signals_nothing_and_reports_this_projects_processes_in_two_tiers() {
    Golden::for_test(module_path!(), "report").checked(&Watch::reported("")).unwrap();
}

#[test]
fn opted_in_stopping_spares_the_owner_shell_claude_its_children_other_projects_and_a_mixed_group() {
    Golden::for_test(module_path!(), "stopping").checked(&Watch::reported(STOPPING)).unwrap();
}

#[test]
fn watch_items_persist_into_obvious_and_the_trail_keeps_no_repeats_failures_or_blind_ticks_within_its_cap() {
    let checkout = tempfile::tempdir().unwrap();
    let settings = CiSettings::try_from(&Config::default()).unwrap();
    let clone = LocalClone::new(checkout.path().to_path_buf(), checkout.path().join(GIT), settings);
    let git_dir = clone.git_dir().as_ref();
    let later = SCENE.replace(LOOP[0], LOOP[1]).replace(JOINED[0], JOINED[1]).replace(RISING[0], RISING[1]);
    let scenes = [Some(SCENE), Some(later.as_str()), Some(later.as_str()), None, None];
    let ticks: Vec<Output> = scenes
        .iter()
        .zip(LATER)
        .map(|(scene, offset)| {
            Watch::patrolled(&FakeMachine::new(scene.as_slice(), CWDS, UPTIME), git_dir, PROMOTING, offset)
        })
        .collect();
    let last = NOW.parse::<DateTime<Utc>>().unwrap() + LATER[4];
    let failure = CliFailure::raised::<Setup>(BROKEN);
    Monitor::noted(&clone, &failure, last);
    Monitor::noted(&clone, &failure, last);
    let recent = Monitor::recent(&clone, last).unwrap();
    let heartbeat = HeartbeatSettings::try_from(&Config::default()).unwrap();
    let report = Monitor::Report.answer(&clone, &heartbeat).unwrap().to_string();
    assert!(ticks.iter().all(|tick| matches!(tick, Output::Flagged(_))) && report == ticks[4].to_string());
    let trail = Output::Listed(Listed::new("")).raised_with(recent).to_string();
    Golden::for_test(module_path!(), "trail").checked(&trail).unwrap();
}
