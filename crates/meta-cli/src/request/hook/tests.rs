use std::path::Path;
use std::time::Instant;

use meta_git::{Layout, Repository};

use super::hook_input::Unreadable;
use super::refusals::Refusals;
use super::{HookEvent, HookInput};
use crate::request::dispatch::Brief;
use crate::request::output::Listed;
use crate::request::project::{CiSettings, Config, Project};
use crate::request::scale_policy::ScalePolicy;

const HOOKS: &str = include_str!("../../../../../plugin/hooks/hooks.json");
const EVENTS: [HookEvent; 4] =
    [HookEvent::SessionStart, HookEvent::PreToolUse, HookEvent::PostToolUse, HookEvent::Stop];
const OUTSIDE: &str =
    r#"{"cwd":"/nonexistent-meta-dir","tool_name":"Bash","tool_input":{"command":"git commit -m x"}}"#;
const GATE_ROWS: [&str; 3] = [
    r#"{"agent_id":"a1","tool_name":"Agent","tool_input":{"prompt":"lease: x y"}}"#,
    r#"{"tool_name":"Agent","tool_input":{}}"#,
    r#"{"tool_input":{"command":"ls"}}"#,
];
const NESTED: &str = "an agent may not start agents";
const UNRESOLVED: &str = "cannot resolve the commit directory";
const REFUSED: [(HookEvent, &str); 2] =
    [(HookEvent::PreToolUse, "\"permissionDecision\":\"deny\""), (HookEvent::Stop, "\"decision\":\"block\"")];

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
fn an_agent_started_by_an_agent_or_without_a_prompt_and_a_call_naming_no_tool_are_refused() {
    let refused = GATE_ROWS.map(|input| {
        let input = serde_json::from_str::<HookInput>(input).unwrap();
        Refusals::new(&input, None).first().map(|refusal| refusal.to_string()).unwrap_or_default()
    });
    assert!(refused[0].contains(NESTED) && refused.iter().all(|refusal| !refusal.is_empty()), "{refused:?}");
}

const HOW_TO_START: &str = "meta claim <task> --agent <name>";
const UNLEASED: &str = "work on it";
const CONFIG: &str = "remote = \"origin\"\n";
const NOT_TEXT: &str = r#"{"tool_name":"Agent","tool_input":{"prompt":["not text"]}}"#;
const DENY: &str = "\"permissionDecision\":\"deny\"";

#[test]
fn an_agent_without_a_lease_line_for_this_project_is_refused_with_how_to_start_one() {
    let dir = tempfile::tempdir().unwrap();
    let working = dir.path().join("working");
    Repository::created(&working, Layout::Working).unwrap();
    std::fs::write(working.join("meta.toml"), CONFIG).unwrap();
    let project = Project::found(&working).unwrap().unwrap();
    let elsewhere = format!("{}{}\nlease: x y", Brief::PROJECT, dir.path().display());
    let refusal = |cwd: &Path, prompt: &str, here: Option<&Project>| {
        let input = serde_json::json!({ "cwd": cwd, "tool_name": "Agent", "tool_input": { "prompt": prompt } });
        let input = serde_json::from_value::<HookInput>(input).unwrap();
        Refusals::new(&input, here).first().map(|refusal| refusal.to_string()).unwrap_or_default()
    };
    let shown = [
        refusal(&working, UNLEASED, Some(&project)),
        refusal(&working, &elsewhere, Some(&project)),
        refusal(dir.path(), UNLEASED, None),
    ];
    let reasons = ["no lease line in the prompt", "is not this session's project", "no meta project at"];
    assert!(shown.iter().zip(reasons).all(|(shown, reason)| shown.contains(reason) && shown.contains(HOW_TO_START)));
    let unparsed = serde_json::from_str::<HookInput>(NOT_TEXT).map_err(Unreadable::from).unwrap_err();
    assert!(HookEvent::PreToolUse.unanswered(unparsed).unwrap().to_string().contains(DENY));
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
fn a_refused_project_denies_tool_use_and_blocks_stop_and_outside_one_only_pre_tool_use_refuses_and_session_start_opens()
{
    let shown = REFUSED.map(|(event, _)| event.refused(Listed::new(REFUSED[0].1)).to_string());
    assert!(shown.iter().zip(REFUSED).all(|(shown, (_, verdict))| shown.contains(verdict)), "{shown:?}");
    let input: HookInput = serde_json::from_str(OUTSIDE).unwrap();
    let refused = [HookEvent::SessionStart, HookEvent::PreToolUse, HookEvent::Stop]
        .map(|event| event.outside_projects(&input).is_some_and(|reply| reply.to_string().contains(UNRESOLVED)));
    assert_eq!((EVENTS.map(HookEvent::opens_session), refused), ([Some(()), None, None, None], [false, true, false]));
}
