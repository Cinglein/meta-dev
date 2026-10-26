use meta_schema::{DueTime, Id, PullRequest, Task};

use super::Flags;
use crate::flag::Flag;
use crate::github_failure::GithubFailure;

const TASK: &str = "task-00000000000000c0";
const DUE: &str = "2026-10-01T00:00:00Z";
const ORPHAN: &str = "16";
const RED: &str = "15";

fn orphan() -> Flag {
    Flag::Orphan(ORPHAN.parse::<PullRequest>().unwrap())
}

fn red() -> Flag {
    Flag::RedUnheld(TASK.parse::<Id<Task>>().unwrap(), RED.parse::<PullRequest>().unwrap())
}

fn stalled() -> Flag {
    Flag::Stalled(TASK.parse::<Id<Task>>().unwrap(), DUE.parse::<DueTime>().unwrap())
}

#[test]
fn since_keeps_only_the_flags_absent_from_the_previous_set_in_order() {
    let now = Flags::new(vec![orphan(), red(), stalled()]);
    let before = Flags::new(vec![red()]);
    assert_eq!(now.since(&before), Flags::new(vec![orphan(), stalled()]));
    assert_eq!(now.since(&now), Flags::default());
    assert_eq!(now.since(&Flags::default()), now);
}

#[test]
fn only_orphans_never_await_judgement() {
    let now = Flags::new(vec![orphan(), red(), Flag::TruncatedOpen, Flag::Unreached(GithubFailure::new("offline"))]);
    assert_eq!(now.awaiting_judgement().to_string().lines().count(), 3);
    assert_eq!(Flags::new(vec![orphan()]).awaiting_judgement().raised(), None);
}

#[test]
fn flags_survive_a_round_trip_through_json() {
    let now = Flags::new(vec![orphan(), red(), stalled(), Flag::TruncatedOpen]);
    let text = serde_json::to_string(&now).unwrap();
    assert_eq!(serde_json::from_str::<Flags>(&text).unwrap(), now);
}
