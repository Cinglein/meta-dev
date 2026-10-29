use chrono::TimeDelta;
use meta_golden::Golden;

use super::{BROKEN, FakeMachine, GIT, Monitor, NOW, STATE, UPTIME, Watch};
use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::local::local_settings::LocalSettings;
use crate::request::local::{LocalClone, LocalFile};
use crate::request::project::{CiSettings, Config};

const UNREADABLE: &str = "[monitor]\ngrace_seconds = 31\n";
const ORPHANS: &str = "stop = [\"orphaned\"]\norphan_seconds = 60";
const REUSED: &str = "  701     1   700      02:00   0:01.00   0.0 node dev-server.js
  801     1   800      04:59   0:01.00   0.0 bash -c while true; do :; done
";
const REUSED_CWDS: &str = "p701\nn/p\np801\nn/p-worktrees/t\n";
const LEADERLESS: &str = r#"{"marked":{"700":{"started":"2026-10-27T09:00:00Z","seen":"2026-10-27T11:55:00Z"},
"800":{"started":"2026-10-27T09:00:00Z","seen":"2026-10-27T11:55:00Z"}},"watched":[]}"#;
const TRAIL: &str = "meta-local/monitor.jsonl";

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
const SCENES: [[&str; 4]; 2] = [[BEFORE, TABLE, CWDS, WALL], [LEADERLESS, REUSED, REUSED_CWDS, ORPHANS]];

#[test]
fn a_mark_is_dropped_when_its_leader_changed_it_went_unseen_left_the_project_or_its_group_id_was_reused() {
    let shown = SCENES.map(|[before, table, cwds, overrides]| {
        let git_dir = tempfile::tempdir().unwrap();
        let state = LocalFile::new(git_dir.path().join(STATE));
        state.replaced(before).unwrap();
        let host = FakeMachine::new(&[table], cwds, UPTIME);
        Watch::patrolled(&host, git_dir.path(), overrides, TimeDelta::zero());
        Watch::shown(&host, &state.text().unwrap())
    });
    Golden::for_test(module_path!(), "marks").checked(&shown.concat()).unwrap();
}

#[test]
fn a_failure_is_noted_once_even_when_the_local_settings_are_unreadable() {
    let checkout = tempfile::tempdir().unwrap();
    LocalFile::new(checkout.path().join(LocalSettings::FILE)).replaced(UNREADABLE).unwrap();
    let settings = CiSettings::try_from(&Config::default()).unwrap();
    let clone = LocalClone::new(checkout.path().to_path_buf(), checkout.path().join(GIT), settings);
    let failure = CliFailure::raised::<Setup>(BROKEN);
    Monitor::noted(&clone, &failure, NOW.parse().unwrap());
    Monitor::noted(&clone, &failure, NOW.parse().unwrap());
    let trail = LocalFile::new(clone.git_dir().as_ref().join(TRAIL)).text().unwrap();
    Golden::for_test(module_path!(), "noted").checked(&trail).unwrap();
}
