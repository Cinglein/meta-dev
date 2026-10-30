use std::path::Path;
use std::time::Instant;

use super::{HookEvent, HookInput};
use crate::request::output::Listed;
use crate::request::project::{CiSettings, Config};
use crate::request::scale_policy::ScalePolicy;

const HOOKS: &str = include_str!("../../../../../plugin/hooks/hooks.json");
const EVENTS: [HookEvent; 4] =
    [HookEvent::SessionStart, HookEvent::PreToolUse, HookEvent::PostToolUse, HookEvent::Stop];
const OUTSIDE: &str = r#"{"cwd":"/nonexistent-meta-dir","tool_input":{"command":"git commit -m x"}}"#;
const UNRESOLVED: &str = "cannot resolve the commit directory";
const ORIGIN: &str = "[remote \"origin\"]\n\turl = https://github.com/o/r\n";
const DENIED: &str = "permissionDecision";
const FROM_OUTSIDE: [(&str, bool); 25] = [
    ("cd ~/meta-development-lead && gh pr merge 1", true),
    ("gh pr merge 1 -R Cinglein/meta-dev", true),
    ("gh pr merge 1 --repo Cinglein/meta-dev", true),
    ("cd /nonexistent-meta-dir; git push --force origin x", true),
    ("git -C /nonexistent-meta-dir push origin main", true),
    ("cd \"a b\" && gh pr merge 1", true),
    ("cd -P /tmp && gh pr merge 1", true),
    ("pushd /tmp && gh pr merge 1", true),
    ("bash -c \"cd /tmp && gh pr merge 1\"", true),
    ("cd $(pwd) && git push --force origin x", true),
    ("gh pr merge 1 -Ro/r", true),
    ("gh pr merge 1 -R=o/r", true),
    ("gh pr merge 1 --repo=o/r", true),
    ("GH_REPO=o/r gh pr merge 1", true),
    ("gh pr merge https://github.com/o/r/pull/1", true),
    ("gh pr merge 1 -R $REPO", true),
    ("gh pr merge 1 -R", true),
    ("ls", false),
    ("cd /tmp && ls", false),
    ("gh pr view 1 -R Cinglein/meta-dev", false),
    ("cp -R a b", false),
    ("chmod -R 755 x", false),
    ("grep -R x .", false),
    ("cd \"a b\" && cp -R a b", false),
    ("gh pr merge 1", false),
];
const RELEASE: &str = "release";
const FEATURE: &str = "feature";
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

const AGENT_ROWS: [(&str, bool); 11] = [
    (r#"{"agent_type":"t","cwd":"/nonexistent-meta-dir","tool_input":{"command":"sed -i p f"}}"#, false),
    (r#"{"agent_id":null,"cwd":"/nonexistent-meta-dir","tool_input":{"command":"sed -i p f"}}"#, false),
    (r#"{"agent_id":"a1","cwd":"/nonexistent-meta-dir","tool_input":{"command":"sed -i p f"}}"#, true),
    (r#"{"cwd":"/nonexistent-meta-dir","tool_input":{"command":"sed -i p f"}}"#, false),
    (r#"{"agent_id":"a1","cwd":"/nonexistent-meta-dir","tool_input":{"command":"git stash"}}"#, true),
    (r#"{"agent_id":"a1","cwd":"/nonexistent-meta-dir","tool_input":{"command":"git push --force origin f"}}"#, true),
    (
        r#"{"agent_id":"a1","cwd":"/nonexistent-meta-dir","tool_input":{"command":"git push --force-with-lease o f"}}"#,
        true,
    ),
    (r#"{"agent_id":"a1","cwd":"/nonexistent-meta-dir","tool_input":{"command":"ls 2>/dev/null"}}"#, true),
    (r#"{"agent_id":"a1","cwd":"/nonexistent-meta-dir","tool_input":{"command":"git rm x && git mv a b"}}"#, false),
    (r#"{"agent_id":"a1","cwd":"/nonexistent-meta-dir","tool_input":{"command":"git push -u origin f"}}"#, false),
    (r#"{"agent_id":"a1","cwd":"/nonexistent-meta-dir","tool_input":{"command":"echo git stash"}}"#, false),
];

#[test]
fn a_subagent_running_a_forbidden_command_is_denied_outside_a_project_and_the_main_session_and_git_rm_are_not() {
    let denied = |input: &str| {
        let input = serde_json::from_str::<HookInput>(input).unwrap();
        HookEvent::PreToolUse.outside_projects(&input).is_some_and(|reply| reply.to_string().contains("deny"))
    };
    assert_eq!(AGENT_ROWS.map(|(input, _)| denied(input)), AGENT_ROWS.map(|(_, denial)| denial));
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

#[test]
fn a_command_outside_every_project_is_judged_by_the_project_it_names_and_by_the_strictest_rules_when_it_names_none() {
    let wrong = FROM_OUTSIDE
        .iter()
        .filter(|(command, expected)| Scene::refused_from(Path::new("/tmp"), command).is_some() != *expected);
    assert_eq!(wrong.map(|(command, _)| *command).collect::<Vec<_>>(), Vec::<&str>::new());
}

struct Scene;

impl Scene {
    fn refused_from(cwd: &Path, command: &str) -> Option<()> {
        let input = serde_json::json!({ "cwd": cwd, "tool_input": { "command": command } }).to_string();
        let input: HookInput = serde_json::from_str(&input).unwrap();
        HookEvent::PreToolUse.answer(&input).to_string().contains(DENIED).then_some(())
    }

    fn founded(dir: &tempfile::TempDir, branch: &str) {
        let config = format!("remote = \"origin\"\nprotected = [\"{branch}\"]\n");
        std::fs::write(dir.path().join("meta.toml"), config).unwrap();
        meta_git::Repository::created(dir.path(), meta_git::Layout::Working).unwrap();
        std::fs::write(dir.path().join(".git/config"), ORIGIN).unwrap();
    }
}

#[test]
fn a_named_project_applies_its_own_protected_branches_and_its_repository_resolves_a_repo_flag() {
    let [project, home] = [tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap()];
    Scene::founded(&project, RELEASE);
    Scene::founded(&home, FEATURE);
    let directory = project.path().display();
    let own = home.path().display();
    let inside = [
        (format!("git -C {directory} push origin {RELEASE}"), true),
        (format!("git -C {directory} push --force origin x"), true),
        (format!("git -C {directory} status"), false),
        (format!("git -C {own} status"), false),
        (format!("cd {own} && git status"), false),
        (format!("cd {directory} && gh pr view 1 -R o/r"), false),
        (format!("cd {directory} && gh pr merge 1 -R o/r"), true),
        (format!("git push origin {FEATURE} && git -C {directory} status"), true),
        (format!("git -C {directory} push origin {FEATURE}"), true),
        ("git status".to_owned(), false),
        ("cd \"a b\" && git push --force origin x".to_owned(), true),
        ("git status && pushd /tmp && gh pr merge 1".to_owned(), true),
        ("git -C /nonexistent-meta-dir status && git push origin main".to_owned(), true),
        ("gh pr view 1 -R o/r".to_owned(), false),
        ("gh pr view 1 -R x/y".to_owned(), false),
        ("gh pr merge 1 -R x/y".to_owned(), true),
    ];
    let wrong =
        inside.iter().filter(|(command, expected)| Scene::refused_from(home.path(), command).is_some() != *expected);
    assert_eq!(wrong.map(|(command, _)| command.as_str()).collect::<Vec<_>>(), Vec::<&str>::new());
    let named = [
        (format!("git -C {directory} push origin {RELEASE}"), true),
        (format!("git -C {directory} status"), false),
        (format!("cd {directory} && git push origin {RELEASE}"), true),
        (format!("cd {directory} && gh pr merge 1 -R o/r"), true),
        (format!("cd {directory} && gh pr view 1 -R o/r"), false),
        (format!("cd {directory} && gh pr view 1 -R x/y"), false),
    ];
    let wrong = named
        .iter()
        .filter(|(command, expected)| Scene::refused_from(Path::new("/tmp"), command).is_some() != *expected);
    assert_eq!(wrong.map(|(command, _)| command.as_str()).collect::<Vec<_>>(), Vec::<&str>::new());
}
