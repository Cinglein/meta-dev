use chrono::TimeDelta;

use super::{FakeMachine, STATE, UPTIME, Watch};
use crate::request::local::LocalFile;

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
    let git_dir = tempfile::tempdir().unwrap();
    let state = LocalFile::new(git_dir.path().join(STATE));
    state.replaced(BEFORE).unwrap();
    let host = FakeMachine::new(&[TABLE], CWDS, UPTIME);
    Watch::patrolled(&host, git_dir.path(), WALL, TimeDelta::zero());
    assert_eq!(host.to_string(), ACTS);
    assert_eq!(state.text().unwrap(), AFTER);
}
