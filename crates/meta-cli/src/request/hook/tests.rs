use std::time::Instant;

use super::reply::{Reply, Verdict};
use super::{HookEvent, HookInput};
use crate::request::output::Listed;
use crate::request::project::{CiSettings, Config};
use crate::request::scale_policy::ScalePolicy;

const HOOKS: &str = include_str!("../../../../../plugin/hooks/hooks.json");
const EVENTS: [HookEvent; 4] =
    [HookEvent::SessionStart, HookEvent::PreToolUse, HookEvent::PostToolUse, HookEvent::Stop];
const OUTSIDE: &str = r#"{"cwd":"/nonexistent-meta-dir","tool_input":{"command":"git commit -m x"}}"#;
const UNRESOLVED: &str = "cannot resolve the commit directory";
const REFUSED: [(HookEvent, &str); 2] =
    [(HookEvent::PreToolUse, "\"permissionDecision\":\"deny\""), (HookEvent::Stop, "\"decision\":\"block\"")];
const REASON: &str = "remote unreadable";
const TOLD: &str = "\"meta-development: remote unreadable\"";

const OUTSIDE_ASK: &str = r#"{"cwd":"/nonexistent-meta-dir","tool_name":"AskUserQuestion","tool_input":{}}"#;
const MODAL_ROWS: [(&str, bool); 4] = [
    (r#"{"tool_name":"AskUserQuestion","tool_input":{}}"#, true),
    (r#"{"agent_id":"a1","tool_name":"AskUserQuestion","tool_input":{}}"#, true),
    (r#"{"tool_name":"Bash","tool_input":{"command":"ls"}}"#, false),
    (r#"{"tool_input":{"command":"ls"}}"#, false),
];

#[test]
fn a_tool_listed_as_modal_in_policy_is_refused_with_or_without_an_agent_and_other_calls_pass() {
    let tools = ScalePolicy::embedded_section(ScalePolicy::tools).unwrap();
    let refusal = |input: &str| serde_json::from_str::<HookInput>(input).unwrap().modal(tools);
    assert_eq!(MODAL_ROWS.map(|(input, _)| refusal(input).is_some()), MODAL_ROWS.map(|(_, modal)| modal));
    assert!(refusal(MODAL_ROWS[0].0).unwrap().to_string().contains("discussion in your reply"));
    let outside = serde_json::from_str::<HookInput>(OUTSIDE_ASK).unwrap();
    assert!(HookEvent::PreToolUse.outside_projects(&outside).is_some_and(|reply| reply.to_string().contains("deny")));
}

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

#[test]
fn a_refused_project_denies_tool_use_and_blocks_stop_and_outside_one_only_pre_tool_use_refuses_and_session_start_opens()
{
    let shown = REFUSED.map(|(event, _)| event.refused(Listed::new(REFUSED[0].1)).to_string());
    assert!(shown.iter().zip(REFUSED).all(|(shown, (_, verdict))| shown.contains(verdict)), "{shown:?}");
    let input: HookInput = serde_json::from_str(OUTSIDE).unwrap();
    let refused = [HookEvent::SessionStart, HookEvent::PreToolUse, HookEvent::Stop]
        .map(|event| event.outside_projects(&input).is_some_and(|reply| reply.to_string().contains(UNRESOLVED)));
    assert_eq!((EVENTS.map(HookEvent::opens_session), refused), ([Some(()), None, None, None], [false, true, false]));
}
