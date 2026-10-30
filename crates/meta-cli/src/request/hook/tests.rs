use std::time::Instant;

use super::{HookEvent, HookInput};
use crate::request::output::Listed;
use crate::request::project::{CiSettings, Config};

const HOOKS: &str = include_str!("../../../../../plugin/hooks/hooks.json");
const EVENTS: [HookEvent; 4] =
    [HookEvent::SessionStart, HookEvent::PreToolUse, HookEvent::PostToolUse, HookEvent::Stop];
const OUTSIDE: &str = r#"{"cwd":"/nonexistent-meta-dir","tool_input":{"command":"git commit -m x"}}"#;
const UNRESOLVED: &str = "cannot resolve the commit directory";
const DENIED: &str = "permissionDecision";
const FROM_OUTSIDE: [(&str, bool); 8] = [
    ("cd ~/meta-development-lead && gh pr merge 1", true),
    ("gh pr merge 1 -R Cinglein/meta-dev", true),
    ("gh pr merge 1 --repo Cinglein/meta-dev", true),
    ("cd /nonexistent-meta-dir; git push --force origin x", true),
    ("git -C /nonexistent-meta-dir push origin main", true),
    ("ls", false),
    ("cd /tmp && ls", false),
    ("gh pr view 1 -R Cinglein/meta-dev", false),
];
const REFUSED: [(HookEvent, &str); 2] =
    [(HookEvent::PreToolUse, "\"permissionDecision\":\"deny\""), (HookEvent::Stop, "\"decision\":\"block\"")];

#[test]
fn hook_timeouts_match_hooks_json_and_bound_every_git_deadline() {
    let hooks: serde_json::Value = serde_json::from_str(HOOKS).unwrap();
    let settings = CiSettings::try_from(&Config::default()).unwrap();
    let configured = EVENTS.map(|event| hooks["hooks"][event.to_string()][0]["hooks"][0]["timeout"].as_u64());
    assert_eq!(configured, EVENTS.map(|event| Some(event.timeout().as_secs())));
    assert!(EVENTS.iter().all(|event| event.budget(&settings).deadline() < Instant::now() + event.timeout()));
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

#[test]
fn a_command_outside_every_project_is_judged_by_the_project_it_names_and_by_the_strictest_rules_when_it_names_none() {
    let refused = |command: &str| {
        let input = serde_json::json!({ "cwd": "/tmp", "tool_input": { "command": command } }).to_string();
        let input: HookInput = serde_json::from_str(&input).unwrap();
        HookEvent::PreToolUse.outside_projects(&input).is_some_and(|reply| reply.to_string().contains(DENIED))
    };
    assert!(FROM_OUTSIDE.iter().all(|(command, expected)| refused(command) == *expected));
}
