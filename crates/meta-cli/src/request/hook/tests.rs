use std::time::Instant;

use super::HookEvent;
use super::reply::{Reply, Verdict};
use crate::request::output::Listed;
use crate::request::project::{CiSettings, Config};

const HOOKS: &str = include_str!("../../../../../plugin/hooks/hooks.json");
const EVENTS: [HookEvent; 4] =
    [HookEvent::SessionStart, HookEvent::PreToolUse, HookEvent::PostToolUse, HookEvent::Stop];
const REFUSED: [(HookEvent, &str); 2] =
    [(HookEvent::PreToolUse, "\"permissionDecision\":\"deny\""), (HookEvent::Stop, "\"decision\":\"block\"")];
const REASON: &str = "remote unreadable";
const TOLD: &str = "\"meta-development: remote unreadable\"";

#[test]
fn hook_timeouts_match_hooks_json_and_bound_every_git_deadline() {
    let hooks: serde_json::Value = serde_json::from_str(HOOKS).unwrap();
    let settings = CiSettings::try_from(&Config::default()).unwrap();
    let configured = EVENTS.map(|event| hooks["hooks"][event.to_string()][0]["hooks"][0]["timeout"].as_u64());
    assert_eq!(configured, EVENTS.map(|event| Some(event.timeout().as_secs())));
    assert!(EVENTS.iter().all(|event| event.budget(&settings).deadline() < Instant::now() + event.timeout()));
}

#[test]
fn a_refused_project_denies_tool_use_and_blocks_stop_with_its_reason_and_a_quiet_allow_says_nothing() {
    let shown = REFUSED.map(|(event, _)| event.refused(Listed::new(REASON)).to_string());
    let told = |(shown, (_, verdict)): (&String, (HookEvent, &str))| shown.contains(verdict) && shown.contains(TOLD);
    assert!(shown.iter().zip(REFUSED).all(told), "{shown:?}");
    assert_eq!(Reply::new(HookEvent::PreToolUse, Verdict::Allow, None).to_string(), "");
}
