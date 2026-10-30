#![allow(clippy::unwrap_used)]

use std::fmt;
use std::io::Write as _;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const META: &str = env!("CARGO_BIN_EXE_meta");
const HOOK: &str = "hook";
const CONFIG: &str = "remote = \"origin\"\nprotected = [\"main\"]\n";
const MERGE: &str = "gh pr merge 1";
const PUSH: &str = "git push --force origin feature";
const ALLOWED: &str = "git push -u origin feature";
const DENY: &str = "\"hookEventName\":\"PreToolUse\",\"permissionDecision\":\"deny\",\"permissionDecisionReason\":\"meta-development: ";
const PUSH_REASON: &str = "meta-development: git push";
const PRE: &str = "pre-tool-use";
const POST: &str = "post-tool-use";
const COMMIT: &str = "git commit -m change";
const STATUS: &str = "git status";
const NO_REFS_LEN: usize = 0;
const AMEND: WorkStep = WorkStep::Amend("rewritten");
const LEAK_COMMIT: WorkStep = WorkStep::Commit("leak");
const FIX_COMMIT: WorkStep = WorkStep::CommitTracked("fix");
const CLEAN_COMMIT: WorkStep = WorkStep::Commit("clean");
const STAGE_AND_COMMIT: &str = "git add -A && git commit -m leak";
const LEAK_FILE: &str = "café.txt";
const LEAK_TEXT: &str = concat!("ok\n/tmp/", "claude-501/x\n\0");
const HOME_TEXT: &str = "WORKDIR /home/app\n/home/runner/x\n";
const HELD_REASON: &str = "café.txt:2: agent scratch path";
const FOREIGN_IDENTITY: &str = "[user]\n\tname = someone\n\temail = someone@example.com\n";
const NOREPLY_IDENTITY: &str = "[user]\n\tname = someone\n\temail = 1+someone@users.noreply.github.com\n";
const GIT_CONFIG: &str = ".git/config";
const BARE_CONFIG: &str = "config";
const ELSEWHERE: &str = "ELSEWHERE";
const FOREIGN_REASON: &str = "someone@example.com is not noreply; git config user.name";
const REFUSED_COMMIT: &str = "git commit -m x && git -C ELSEWHERE commit -m y";
const NEW_FEATURE: WorkStep = WorkStep::Branch("feature");
const HEAD: &str = "HEAD";
const FEATURE_BRANCH: &str = "feature";
const REJECTED: &str = "rejected";
const SESSION_START: &str = "session-start";
const PATIENT: Duration = Duration::from_secs(2);
const POLL: Duration = Duration::from_millis(2);
const ENV_FILE: &str = ".env";
const SESSION_FIELD: &str = r#""session_id":"ABCDEF12-34","#;
const WATCHERS: &str = ".git/meta-ci";
const NO_TASKS: &str = "Tasks: 0 unfinished";
const SESSION_ACTOR: &str = "claude-abcdef1234";
const SESSION_STARTED: &str = "meta-sessions/ABCDEF1234/started";
const ENV_LINE: &str = "export META_ACTOR=claude-abcdef1234\nexport META_SESSION=ABCDEF12-34\n";
const HANGING_UPLOAD: &str = "[remote \"origin\"]\n\tuploadpack = sh -c 'sleep 5' x\n";
const META_TOML: &str = "meta.toml";
const QUICK_GIT: &str = "[ci]\ngit_millis = 20\n";
const SHORT_GIT: &str = "[ci]\ngit_millis = 250\n";
const SLOW_RECEIVE: &str = "[remote \"origin\"]\n\treceivepack = sh -c 'sleep 0.3 && exec git receive-pack $1' x\n";
const UNREADABLE: &str = "meta/log unreadable";
const OPEN: [&str; 4] = ["open", "--division", "features-and-maintenance", "text"];
const ROOT: &str = "ROOT";
const PIDS: &str = "pids";
const IGNORING_HUP: &str = "trap '' HUP; exec \"$0\" \"$@\"";
const INTERRUPTED: &str = "SIGINT";
const HELPERS: &str = r#"[remote "origin"]
	receivepack = "sh -c 'trap \"\" TERM; touch ROOT/hold && echo $$ >> ROOT/pids && sleep 5' x"
	uploadpack = "sh -c 'test -e ROOT/hold && echo $$ >> ROOT/pids && sleep 5 || exec git upload-pack \"$1\"' x"
"#;

use crate::fixture::{Fixture, WorkStep};
use crate::meta_command::MetaCommand;

struct Config;

impl Config {
    fn configure(file: &Path, section: &str) {
        std::fs::OpenOptions::new().append(true).open(file).unwrap().write_all(section.as_bytes()).unwrap();
    }
}

struct Repo {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
}

impl Repo {
    fn new() -> Self {
        let origin = tempfile::tempdir().unwrap();
        let clone = tempfile::tempdir().unwrap();
        Fixture::bare(origin.path(), "main");
        let cloned = Fixture::working(clone.path(), origin.path(), "main");
        std::fs::write(clone.path().join("meta.toml"), CONFIG).unwrap();
        std::fs::create_dir_all(clone.path().join("tooling")).unwrap();
        std::fs::write(clone.path().join("tooling/policy.toml"), "").unwrap();
        cloned.committed("start");
        Self { origin, clone }
    }

    fn hook(&self, event: &str, command: &str, cwd: &Path) -> String {
        let child = self.started(event, command, cwd, &mut MetaCommand::tagged(&[HOOK, event]));
        String::from_utf8_lossy(&child.wait_with_output().unwrap().stdout).into_owned()
    }

    fn started(&self, event: &str, command: &str, cwd: &Path, meta: &mut Command) -> Child {
        let session = if event == SESSION_START { SESSION_FIELD } else { "" };
        let input = format!(
            r#"{{{session}"cwd":{},"tool_input":{{"command":{}}}}}"#,
            serde_json::to_string(cwd).unwrap(),
            serde_json::to_string(&command.replace(ELSEWHERE, &self.origin.path().display().to_string())).unwrap()
        );
        let meta = meta.env_remove("META_ACTOR").env("CLAUDE_ENV_FILE", self.clone.path().join(ENV_FILE));
        let piped = meta.env("TMPDIR", self.clone.path()).stdin(Stdio::piped()).stdout(Stdio::piped());
        let mut child = piped.spawn().unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        child
    }

    fn interrupted(&self) -> bool {
        let root = self.clone.path().display().to_string();
        Config::configure(&self.clone.path().join(GIT_CONFIG), &HELPERS.replace(ROOT, &root));
        let mut meta = Command::new("sh");
        meta.args(["-c", IGNORING_HUP, META]).args(OPEN).current_dir(self.clone.path()).env("META_ACTOR", "writer");
        let mut meta = meta.process_group(0).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        let polls = || {
            let until = Instant::now() + PATIENT;
            std::iter::repeat_with(|| std::thread::sleep(POLL)).take_while(move |()| Instant::now() < until)
        };
        let listed = || std::fs::read_to_string(self.clone.path().join(PIDS)).unwrap_or_default();
        let started = polls().find(|()| !listed().is_empty());
        let signal = |name: &str, target: String| Command::new("kill").args([name, "--", &target]).status().unwrap();
        cfg!(target_os = "linux").then(|| signal("-HUP", meta.id().to_string()));
        signal("-INT", format!("-{}", meta.id()));
        let alive = |pid: &str| Command::new("kill").args(["-0", pid]).status().unwrap().success();
        let status = meta.wait().map(|status| status.to_string()).unwrap_or_default();
        status.contains(INTERRUPTED) && started.is_some() && polls().any(|()| !listed().split_whitespace().any(alive))
    }
}

#[test]
fn pre_tool_use_denies_merges_force_pushes_and_commits_by_foreign_emails_wherever_they_run() {
    let repo = Repo::new();
    let home = repo.clone.path();
    let elsewhere = repo.origin.path();
    Config::configure(&home.join(GIT_CONFIG), NOREPLY_IDENTITY);
    Config::configure(&elsewhere.join(BARE_CONFIG), FOREIGN_IDENTITY);
    let push = repo.hook(PRE, PUSH, home);
    assert!(repo.hook(PRE, MERGE, home).contains(DENY) && push.contains(DENY) && push.contains(PUSH_REASON));
    assert!(repo.hook(PRE, REFUSED_COMMIT, home).contains(&format!("{DENY}{FOREIGN_REASON}")));
    assert_eq!(repo.hook(PRE, ALLOWED, home), "");
}

#[test]
fn post_tool_use_pushes_unprotected_branches_and_reports_git_errors() {
    let repo = Repo::new();
    let home = repo.clone.path();
    repo.hook(POST, COMMIT, home);
    let git = Fixture::opened(home);
    let origin = Fixture::opened(repo.origin.path());
    git.stepped(&NEW_FEATURE);
    std::fs::write(home.join(LEAK_FILE), LEAK_TEXT).unwrap();
    git.stepped(&WorkStep::StageAll);
    git.stepped(&LEAK_COMMIT);
    repo.hook(POST, STATUS, home);
    let held = repo.hook(POST, STAGE_AND_COMMIT, home);
    assert!(held.contains(HELD_REASON) && origin.refs_len() == NO_REFS_LEN);
    std::fs::write(home.join(LEAK_FILE), HOME_TEXT).unwrap();
    git.stepped(&FIX_COMMIT);
    let after_fix = repo.hook(POST, COMMIT, home);
    let by_hand = repo.hook(PRE, ALLOWED, home);
    assert!(after_fix.contains(HELD_REASON) && by_hand.contains(DENY) && by_hand.contains(HELD_REASON));
    git.stepped(&WorkStep::Rewind);
    git.stepped(&CLEAN_COMMIT);
    repo.hook(POST, COMMIT, home);
    assert_eq!(origin.tip(FEATURE_BRANCH), git.tip(HEAD));
    git.stepped(&AMEND);
    assert!(repo.hook(POST, COMMIT, home).contains(REJECTED) && !repo.clone.path().join(WATCHERS).exists());
    assert!(repo.interrupted(), "an interrupted write left a git helper running");
    let slow = Repo::new();
    let slow_home = slow.clone.path();
    Fixture::opened(slow_home).stepped(&NEW_FEATURE);
    Config::configure(&slow_home.join(GIT_CONFIG), SLOW_RECEIVE);
    Config::configure(&slow_home.join(META_TOML), SHORT_GIT);
    slow.hook(POST, COMMIT, slow_home);
    assert_eq!(Fixture::opened(slow.origin.path()).tip(FEATURE_BRANCH), Fixture::opened(slow_home).tip(HEAD));
}

#[test]
fn session_start_shows_state_and_names_the_session_actor() {
    let repo = Repo::new();
    let home = repo.clone.path();
    let shown = repo.hook(SESSION_START, COMMIT, home);
    assert!(shown.contains(NO_TASKS) && shown.contains(SESSION_ACTOR));
    assert_eq!(std::fs::read_to_string(home.join(ENV_FILE)).unwrap(), ENV_LINE);
    Config::configure(&home.join(GIT_CONFIG), HANGING_UPLOAD);
    Config::configure(&home.join(META_TOML), QUICK_GIT);
    let started = Instant::now();
    let shown = repo.hook(SESSION_START, COMMIT, home);
    assert!(home.join(SESSION_STARTED).exists() && started.elapsed() < PATIENT && shown.contains(UNREADABLE));
}

const REAL_POLICY: &str = include_str!("../../../../tooling/policy.toml");

#[test]
fn a_subagent_running_a_forbidden_command_is_denied_through_the_real_policy_and_the_main_session_is_not() {
    let repo = Repo::new();
    std::fs::write(repo.clone.path().join("tooling/policy.toml"), REAL_POLICY).unwrap();
    let ran = |agent: &str| {
        let input = format!(
            "{{{agent}\"cwd\":{},\"tool_input\":{{\"command\":\"sed -i p f\"}}}}",
            serde_json::to_string(repo.clone.path()).unwrap()
        );
        let mut child = MetaCommand::tagged(&[HOOK, PRE]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        String::from_utf8_lossy(&child.wait_with_output().unwrap().stdout).into_owned()
    };
    assert!(ran("\"agent_id\":\"a1\",").contains(DENY) && ran("").is_empty());
}

#[test]
fn pre_tool_use_denies_input_it_cannot_parse() {
    let mut child = MetaCommand::tagged(&[HOOK, PRE]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(b"{not json").unwrap();
    assert!(String::from_utf8_lossy(&child.wait_with_output().unwrap().stdout).contains(DENY));
}

impl fmt::Display for Repo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.clone.path().display())
    }
}
