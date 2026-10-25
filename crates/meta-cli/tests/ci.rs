#![allow(clippy::unwrap_used)]

use std::fmt;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

const META: &str = env!("CARGO_BIN_EXE_meta");
const CONFIG: &str =
    "remote = \"origin\"\nprotected = [\"main\"]\n[ci]\nprogram = \"gh\"\npoll_millis = 40\ntimeout_millis = 1500\n";
const FAKE_GH: &str = r#"#!/bin/sh
count="$FAKE_GH_DIR/count"
n=$(cat "$count" 2>/dev/null || echo 0)
case "$1 $2" in
"run list")
  n=$((n + 1)); echo "$n" > "$count"
  if [ "$FAKE_GH_SCENARIO" = none ]; then echo '[]'
  elif [ "$n" -lt 4 ]; then echo '[{"databaseId":7,"status":"in_progress","conclusion":""}]'
  else echo "[{\"databaseId\":7,\"status\":\"completed\",\"conclusion\":\"$FAKE_GH_SCENARIO\"}]"; fi ;;
"run view")
  if [ "$FAKE_GH_SCENARIO" = failure ]; then echo '{"jobs":[{"name":"lint","conclusion":"failure"},{"name":"test","conclusion":""}]}'
  else echo '{"jobs":[{"name":"test","conclusion":"success"}]}'; fi ;;
esac
"#;
const SHELL: &str = "/bin/sh";
const SESSION: &str = "ABCDEF12-34";
const FAILURE: &str = "failure";
const SUCCESS: &str = "success";
const NONE: &str = "none";
const COMMIT: &str = "git commit -m change";
const PRE: &str = "pre-tool-use";
const STOP: &str = "stop";
const BLOCK: &str = "\"decision\":\"block\"";
const JOB_FAILED: &str = "lint failed after";
const RUN_FAILED: &str = "failed after";
const PASSED: &str = "passed after";
const TIMED_OUT: &str = "timed-out after";
const COMPLETIONS: [&str; 3] = [": passed after", ": failed after", ": timed-out after"];

struct Git;

impl Git {
    const IDENTITY: [&'static str; 6] =
        ["-c", "user.name=t", "-c", "user.email=t@meta.invalid", "-c", "init.defaultBranch=main"];

    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }
}

struct Waited(String);

struct Repo {
    _origin: tempfile::TempDir,
    clone: tempfile::TempDir,
    scenario: &'static str,
}

impl Repo {
    fn new(scenario: &'static str) -> Self {
        let origin = tempfile::tempdir().unwrap();
        let clone = tempfile::tempdir().unwrap();
        Git::run(origin.path(), &["init", "-q", "--bare"]);
        Git::run(clone.path(), &["init", "-q"]);
        Git::run(clone.path(), &["remote", "add", "origin", origin.path().to_str().unwrap()]);
        std::fs::write(clone.path().join("meta.toml"), CONFIG).unwrap();
        std::fs::create_dir(clone.path().join("bin")).unwrap();
        std::fs::write(clone.path().join("bin/gh"), FAKE_GH).unwrap();
        std::fs::set_permissions(clone.path().join("bin/gh"), std::fs::metadata(SHELL).unwrap().permissions()).unwrap();
        Git::run(clone.path(), &["switch", "-q", "-c", "feature"]);
        Git::run(clone.path(), &["commit", "-q", "--allow-empty", "-m", "change"]);
        Self { _origin: origin, clone, scenario }
    }

    fn hook(&self, event: &str, command: &str) -> String {
        let input = format!(
            r#"{{"session_id":"ABCDEF12-34","cwd":{},"tool_input":{{"command":{}}}}}"#,
            serde_json::to_string(self.clone.path()).unwrap(),
            serde_json::to_string(command).unwrap()
        );
        let path = format!("{}:{}", self.clone.path().join("bin").display(), std::env::var("PATH").unwrap_or_default());
        let mut child = Command::new(META)
            .args(["hook", event])
            .env("PATH", path)
            .env("FAKE_GH_DIR", self.clone.path())
            .env("FAKE_GH_SCENARIO", self.scenario)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        String::from_utf8_lossy(&child.wait_with_output().unwrap().stdout).into_owned()
    }

    fn ci_wait(&self) -> Waited {
        let output = Command::new(META)
            .arg("ci-wait")
            .current_dir(self.clone.path())
            .env("META_SESSION", SESSION)
            .output()
            .unwrap();
        Waited(format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)))
    }

    fn settled(&self) -> String {
        let events = self.clone.path().join(".git/meta-ci/ABCDEF1234/events");
        std::iter::repeat_with(|| {
            std::thread::sleep(Duration::from_millis(50));
            std::fs::read_to_string(&events).unwrap_or_default()
        })
        .take(60)
        .find(|text| COMPLETIONS.iter().any(|completion| text.contains(completion)))
        .unwrap_or_default()
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.clone.path().display())
    }
}

#[test]
fn ci_wait_returns_at_the_first_failure_and_stop_stays_blocked() {
    let repo = Repo::new(FAILURE);
    repo.hook("post-tool-use", COMMIT);
    assert!(repo.ci_wait().0.contains(JOB_FAILED) && repo.hook(STOP, COMMIT).contains(BLOCK));
    assert!(repo.settled().contains(RUN_FAILED) && repo.hook(PRE, COMMIT).contains(RUN_FAILED));
    assert!(!repo.hook(PRE, COMMIT).contains(RUN_FAILED));
}

#[test]
fn a_passing_run_reports_only_completion_and_lets_the_agent_stop() {
    let repo = Repo::new(SUCCESS);
    repo.hook("post-tool-use", COMMIT);
    assert!(repo.ci_wait().0.contains(PASSED) && !repo.hook(STOP, COMMIT).contains(BLOCK));
    assert_eq!(repo.settled().lines().filter(|line| line.contains(PASSED)).count(), repo.settled().lines().count());
}

#[test]
fn a_run_that_never_appears_times_out_and_says_so() {
    let repo = Repo::new(NONE);
    repo.hook("post-tool-use", COMMIT);
    assert!(repo.settled().contains(TIMED_OUT));
}
