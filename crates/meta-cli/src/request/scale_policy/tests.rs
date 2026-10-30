use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use meta_schema::{Block, Blocker, DueTime, ExternalKind};

use super::ScalePolicy;

const MISSING: &str = "tooling/policy.toml";
const TOOLING: &str = "tooling";
const ASKED: &str = "[liveness]\nreply_within = 60000\n";
const NOW: &str = "2026-10-29T12:00:00Z";
const TASK: &str = "task-00000000000000c0";
const WINDOW: TimeDelta = TimeDelta::minutes(1);

const POLICY: &str = "[domains.\"features-and-maintenance/tracker\"]\npaths = [\"x\"]\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"lost\"\nmeaning = \"no map\"\nparent = \"critical\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"slow\"\nmeaning = \"late\"\nparent = \"major\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"typo\"\nmeaning = \"text\"\nparent = \"minor\"\n";
const NAV: &str = "features-and-maintenance/nav";
const KNOWN: [&str; 3] = [NAV, "features-and-maintenance/tracker", "architecture-and-planning"];
const UNKNOWN: &str = "features-and-maintenance/nowhere";
const UNMAPPED: &str = "maps onto no level";
const QUEUED: &str = "[millis]\nqueue_pace = 60000\n[queue]\nlanders = [\"claude-lead\"]\n";
const LANDER: &str = "claude-lead";

#[test]
fn scales_and_known_divisions_are_read_from_policy_and_a_missing_one_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let missing = ScalePolicy::read(dir.path()).unwrap_err().to_string();
    assert!(missing.contains(MISSING) && missing.contains("copy meta-dev's tooling/policy.toml"));
    std::fs::create_dir(dir.path().join("tooling")).unwrap();
    std::fs::write(dir.path().join(ScalePolicy::FILE), POLICY).unwrap();
    let nav = NAV.parse().unwrap();
    let policy = ScalePolicy::read(dir.path()).unwrap();
    assert_eq!(policy.scales().owner(&nav), Some(&nav));
    assert!(KNOWN.iter().all(|known| policy.known(&known.parse().unwrap()).is_ok()));
    assert!(policy.known(&UNKNOWN.parse().unwrap()).is_err());
    std::fs::write(dir.path().join(ScalePolicy::FILE), POLICY.replace("critical", "nope")).unwrap();
    assert!(ScalePolicy::read(dir.path()).unwrap_err().to_string().contains(UNMAPPED));
}

#[test]
fn the_queue_reads_its_pace_floor_and_its_landers_from_policy() {
    let policy: ScalePolicy = toml::from_str(QUEUED).unwrap();
    let floor = [policy.millis().queue_floor().ok(), ScalePolicy::default().millis().queue_floor().ok()];
    assert_eq!(floor, [Some(Duration::from_secs(60)), None]);
    assert!(policy.queue().lander(LANDER.parse().ok()).is_some());
}

#[test]
fn a_wait_without_its_own_due_time_is_due_one_reply_window_from_now() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(TOOLING)).unwrap();
    std::fs::write(dir.path().join(ScalePolicy::FILE), ASKED).unwrap();
    let liveness = ScalePolicy::read(dir.path()).unwrap().liveness();
    let now = NOW.parse::<DateTime<Utc>>().unwrap();
    let block = Block::new(TASK.parse().unwrap(), Blocker::External(ExternalKind::Ci));
    let own = DueTime::from(now + WINDOW + WINDOW);
    let due = [block.clone(), block.reply_by(own)].map(|block| liveness.asked(block, now).reply_due());
    assert_eq!(due, [Some(DueTime::from(now + WINDOW)), Some(own)]);
}

#[test]
fn the_ci_run_limit_is_read_from_policy_and_is_five_minutes_in_the_embedded_policy() {
    let policy: ScalePolicy = toml::from_str("[liveness]\nci_run_within = 60000\n").unwrap();
    let embedded = ScalePolicy::embedded().unwrap().liveness();
    let unset = ScalePolicy::default().liveness().ci_run_within().unwrap_err().to_string();
    assert_eq!(policy.liveness().ci_run_within().unwrap(), Duration::from_secs(60));
    assert_eq!(embedded.ci_run_within().unwrap(), Duration::from_secs(300));
    assert!(unset.contains("[liveness] ci_run_within is missing"));
}
