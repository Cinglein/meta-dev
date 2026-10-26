#![allow(clippy::unwrap_used)]

use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const META: &str = env!("CARGO_BIN_EXE_meta");
const CONFIG: &str = "remote = \"origin\"\nprotected = [\"main\"]\n";
const BLOCKED: &str = "git push --force origin feature";
const ALLOWED: &str = "git push -u origin feature";
const DENY: &str = "\"permissionDecision\":\"deny\"";
const PRE: &str = "pre-tool-use";
const POST: &str = "post-tool-use";
const COMMIT: &str = "git commit -m change";
const AMEND: [&str; 6] = ["commit", "-q", "--amend", "--allow-empty", "-m", "rewritten"];
const NEW_FEATURE: [&str; 4] = ["switch", "-q", "-c", "feature"];
const MAIN: &str = "main";
const HEAD: &str = "HEAD";
const FEATURE_BRANCH: &str = "feature";
const REJECTED: &str = "rejected";
const SESSION_START: &str = "session-start";
const NO_TASKS: &str = "Tasks: 0 unfinished";
const SESSION_ACTOR: &str = "claude-abcdef1234";
const ENV_LINE: &str = "export META_ACTOR=claude-abcdef1234\nexport META_SESSION=ABCDEF12-34\n";

struct Git;

impl Git {
    const IDENTITY: [&'static str; 6] =
        ["-c", "user.name=t", "-c", "user.email=t@meta.invalid", "-c", "init.defaultBranch=main"];

    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }

    fn link_origin(dir: &Path, url: &str) {
        let remote = format!("[remote \"origin\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n");
        std::fs::OpenOptions::new()
            .append(true)
            .open(dir.join(".git/config"))
            .unwrap()
            .write_all(remote.as_bytes())
            .unwrap();
    }

    fn head(dir: &Path, reference: &str) -> Revision {
        Revision(String::from_utf8_lossy(&Self::run(dir, &["rev-parse", reference]).stdout).into_owned())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Revision(String);

struct Repo {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
}

impl Repo {
    fn new() -> Self {
        let origin = tempfile::tempdir().unwrap();
        let clone = tempfile::tempdir().unwrap();
        Git::run(origin.path(), &["init", "-q", "--bare"]);
        Git::run(clone.path(), &["init", "-q"]);
        Git::link_origin(clone.path(), origin.path().to_str().unwrap());
        std::fs::write(clone.path().join("meta.toml"), CONFIG).unwrap();
        Git::run(clone.path(), &["commit", "-q", "--allow-empty", "-m", "start"]);
        Self { origin, clone }
    }

    fn hook(&self, event: &str, command: &str, env_file: &Path) -> String {
        let input = format!(
            r#"{{"session_id":"ABCDEF12-34","cwd":{},"tool_input":{{"command":{}}}}}"#,
            serde_json::to_string(self.clone.path()).unwrap(),
            serde_json::to_string(command).unwrap()
        );
        let mut child = Command::new(META)
            .args(["hook", event])
            .env_remove("META_ACTOR")
            .env("CLAUDE_ENV_FILE", env_file)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        String::from_utf8_lossy(&child.wait_with_output().unwrap().stdout).into_owned()
    }

    fn env_file(&self) -> PathBuf {
        self.clone.path().join(".env")
    }
}

#[test]
fn pre_tool_use_answers_a_refused_command_with_a_denial_and_an_allowed_one_with_nothing() {
    let repo = Repo::new();
    let env = repo.env_file();
    assert!(repo.hook(PRE, BLOCKED, &env).contains(DENY) && repo.hook(PRE, ALLOWED, &env).is_empty());
}

#[test]
fn post_tool_use_pushes_unprotected_branches_and_reports_git_errors() {
    let repo = Repo::new();
    let env = repo.env_file();
    repo.hook(POST, COMMIT, &env);
    assert_ne!(Git::head(repo.origin.path(), MAIN), Git::head(repo.clone.path(), HEAD));
    Git::run(repo.clone.path(), &NEW_FEATURE);
    repo.hook(POST, COMMIT, &env);
    assert_eq!(Git::head(repo.origin.path(), FEATURE_BRANCH), Git::head(repo.clone.path(), HEAD));
    Git::run(repo.clone.path(), &AMEND);
    assert!(repo.hook(POST, COMMIT, &env).contains(REJECTED));
}

#[test]
fn session_start_shows_state_and_names_the_session_actor() {
    let repo = Repo::new();
    let env = repo.env_file();
    let shown = repo.hook(SESSION_START, COMMIT, &env);
    assert!(shown.contains(NO_TASKS) && shown.contains(SESSION_ACTOR));
    assert_eq!(std::fs::read_to_string(env).unwrap(), ENV_LINE);
}

impl fmt::Display for Repo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.clone.path().display())
    }
}
