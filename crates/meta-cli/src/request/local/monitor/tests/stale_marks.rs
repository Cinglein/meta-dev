use chrono::TimeDelta;

use super::{BROKEN, FakeMachine, Monitor, NOW, STATE, UPTIME, Watch};
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::local::local_settings::LocalSettings;
use crate::request::local::{LocalClone, LocalFile};
use crate::request::tests::Repository;

const UNREADABLE: &str = "[monitor]\ngrace_seconds = 31\n";
const ORPHANS: &str = "stop = [\"orphaned\"]\norphan_seconds = 60";
const REUSED: &str = "  701     1   700      02:00   0:01.00   0.0 node dev-server.js
  801     1   800      04:59   0:01.00   0.0 bash -c while true; do :; done
";
const REUSED_CWDS: &str = "p701\nn/p\np801\nn/p-worktrees/t\n";
const LEADERLESS: &str = r#"{"marked":{"700":{"started":"2026-10-27T09:00:00Z","seen":"2026-10-27T11:55:00Z"},
"800":{"started":"2026-10-27T09:00:00Z","seen":"2026-10-27T11:55:00Z"}},"watched":[]}"#;
const UNMARKED: &str =
    r#"{"marked":{"800":{"started":"2026-10-27T09:00:00Z","seen":"2026-10-27T12:00:00Z"}},"watched":[]}"#;
const LOOKED: &str = "cwds 701,801\n-TERM -800\npause 5s\n";
const TRAIL: &str = "meta-local/monitor.jsonl";
const TRAIL_LEN: usize = 1;

const WALL: &str = "stop = [\"wall-time\"]";
const TABLE: &str = "  700   650   700   10:00:00   0:01.00   0.0 -zsh
  710   650   710   09:00:00   0:01.00   0.0 sleep 99999
  720   650   720   09:00:00   0:01.00   0.0 sleep 88888
  730   650   730   09:00:00   0:01.00   0.0 sleep 77777
";
const CWDS: &str = "p700\nn/p\np710\nn/p-worktrees/t\np720\nn/p-worktrees/t\np730\nn/other\n";
const BEFORE: &str = r#"{"marked":{"700":{"started":"2026-10-27T11:50:00Z","seen":"2026-10-27T11:55:00Z"},
"710":{"started":"2026-10-27T03:00:00Z","seen":"2026-10-27T11:40:00Z"},
"720":{"started":"2026-10-27T03:00:00Z","seen":"2026-10-27T11:55:00Z"},
"730":{"started":"2026-10-27T03:00:00Z","seen":"2026-10-27T11:55:00Z"}},"watched":[]}"#;
const AFTER: &str =
    r#"{"marked":{"720":{"started":"2026-10-27T03:00:00Z","seen":"2026-10-27T12:00:00Z"}},"watched":[]}"#;
const ACTS: &str = "cwds 700,710,720,730\n-TERM -720\npause 5s\n";

#[test]
fn a_remembered_mark_is_dropped_when_its_leader_changed_it_went_unseen_or_left_the_project() {
    let git_dir = Repository::new();
    let state = LocalFile::new(git_dir.store().entry(STATE));
    state.replaced(BEFORE).unwrap();
    let host = FakeMachine::new(&[TABLE], CWDS, UPTIME);
    Watch::patrolled(&host, git_dir.store(), WALL, TimeDelta::zero());
    assert_eq!(host.to_string(), ACTS);
    assert_eq!(state.text().unwrap(), AFTER);
}

#[test]
fn a_failure_is_noted_once_even_when_the_local_settings_are_unreadable() {
    let checkout = Repository::new();
    LocalFile::new(checkout.path().join(LocalSettings::FILE)).replaced(UNREADABLE).unwrap();
    let clone = LocalClone::new(checkout.path().to_path_buf(), checkout.store().clone());
    let failure = CliFailure::raised::<Setup>(BROKEN);
    Monitor::noted(&clone, &failure, NOW.parse().unwrap());
    Monitor::noted(&clone, &failure, NOW.parse().unwrap());
    let trail = LocalFile::new(clone.store().entry(TRAIL)).text().unwrap();
    assert_eq!(trail.lines().count(), TRAIL_LEN);
    assert!(trail.contains(BROKEN));
}

#[test]
fn a_leaderless_mark_is_dropped_when_the_group_id_now_holds_a_process_started_after_it_was_last_seen() {
    let git_dir = Repository::new();
    let state = LocalFile::new(git_dir.store().entry(STATE));
    state.replaced(LEADERLESS).unwrap();
    let host = FakeMachine::new(&[REUSED], REUSED_CWDS, UPTIME);
    Watch::patrolled(&host, git_dir.store(), ORPHANS, TimeDelta::zero());
    assert_eq!(host.to_string(), LOOKED);
    assert_eq!(state.text().unwrap(), UNMARKED);
}
