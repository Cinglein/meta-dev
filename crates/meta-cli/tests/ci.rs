#![allow(clippy::unwrap_used)]

use std::fmt;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

const META: &str = env!("CARGO_BIN_EXE_meta");
const CONFIG: &str = "remote = \"origin\"\nprotected = [\"main\"]\n[ci]\nprogram = \"gh\"\npoll_millis = 10\n";
const PATIENT: &str = "timeout_millis = 300\n";
const IMPATIENT: &str = "timeout_millis = 30\n";
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
  if [ "$3" = --job ]; then cat "$FAKE_GH_DIR/bin/failed.log"
  elif [ "$FAKE_GH_SCENARIO" = failure ]; then echo '{"jobs":[{"databaseId":11,"name":"ci","conclusion":"failure","steps":[{"name":"Checkout","conclusion":"success"},{"name":"cargo xtask ci","conclusion":"failure"}]},{"databaseId":12,"name":"docs","conclusion":""}]}'
  else echo '{"jobs":[{"databaseId":11,"name":"ci","conclusion":"success"}]}'; fi ;;
esac
"#;
const FAILED_LOG: &str = "ci\tcargo xtask ci\t2026-10-25T06:40:01.0000000Z    Compiling meta-cli v0.0.0 (/home/runner/work/meta-development/crates/meta-cli)
ci\tcargo xtask ci\t2026-10-25T06:40:09.0000000Z running 3 tests
ci\tcargo xtask ci\t2026-10-25T06:40:09.1000000Z test claims_are_taken_over ... ok
ci\tcargo xtask ci\t2026-10-25T06:40:09.2000000Z test done_addresses_targets ... FAILED
ci\tcargo xtask ci\t2026-10-25T06:40:09.3000000Z
ci\tcargo xtask ci\t2026-10-25T06:40:09.3000000Z ---- done_addresses_targets stdout ----
ci\tcargo xtask ci\t2026-10-25T06:40:09.3000000Z thread 'done_addresses_targets' panicked at crates/meta-state/src/state/tests.rs:42:5:
ci\tcargo xtask ci\t2026-10-25T06:40:09.3000000Z assertion `left == right` failed
ci\tcargo xtask ci\t2026-10-25T06:40:09.3000000Z   left: None
ci\tcargo xtask ci\t2026-10-25T06:40:09.3000000Z  right: Some(task-0123456789abcdef)
ci\tcargo xtask ci\t2026-10-25T06:40:09.4000000Z test result: FAILED. 2 passed; 1 failed; 0 ignored
ci\tcargo xtask ci\t2026-10-25T06:40:09.5000000Z ##[error]Process completed with exit code 101.
";
const SHELL: &str = "/bin/sh";
const SETTLE_POLL: Duration = Duration::from_millis(10);
const SETTLE_LENGTH: usize = 300;
const SESSION: &str = "ABCDEF12-34";
const FAILURE: &str = "failure";
const SUCCESS: &str = "success";
const NONE: &str = "none";
const COMMIT: &str = "git commit -m change";
const PRE: &str = "pre-tool-use";
const STOP: &str = "stop";
const BLOCK: &str = "\"decision\":\"block\"";
const JOB_FAILED: &str = "ci failed after";
const CI_LOG_HINT: &str = "run `meta ci-log`";
const STEP_LISTED: &str = "s: ci / cargo xtask ci; run `meta ci-log`";
const PANIC_BLOCK: &str =
    "panicked at crates/meta-state/src/state/tests.rs:42:5:\nassertion `left == right` failed\n  left: None";
const COMPILING: &str = "Compiling";
const CI_WAIT: &str = "ci-wait";
const CI_LOG: &str = "ci-log";
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

    fn link_origin(dir: &Path, url: &str) {
        let remote = format!("[remote \"origin\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n");
        std::fs::OpenOptions::new()
            .append(true)
            .open(dir.join(".git/config"))
            .unwrap()
            .write_all(remote.as_bytes())
            .unwrap();
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
        Git::run(clone.path(), &["init", "-q", "-b", "feature"]);
        Git::link_origin(clone.path(), origin.path().to_str().unwrap());
        let timeout = if scenario == NONE { IMPATIENT } else { PATIENT };
        std::fs::write(clone.path().join("meta.toml"), format!("{CONFIG}{timeout}")).unwrap();
        std::fs::create_dir(clone.path().join("bin")).unwrap();
        std::fs::write(clone.path().join("bin/gh"), FAKE_GH).unwrap();
        std::fs::write(clone.path().join("bin/failed.log"), FAILED_LOG).unwrap();
        std::fs::set_permissions(clone.path().join("bin/gh"), std::fs::metadata(SHELL).unwrap().permissions()).unwrap();
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

    fn session_command(&self, command: &str) -> Waited {
        let output = Command::new(META)
            .arg(command)
            .current_dir(self.clone.path())
            .env("META_SESSION", SESSION)
            .env(
                "PATH",
                format!("{}:{}", self.clone.path().join("bin").display(), std::env::var("PATH").unwrap_or_default()),
            )
            .env("FAKE_GH_DIR", self.clone.path())
            .env("FAKE_GH_SCENARIO", self.scenario)
            .output()
            .unwrap();
        Waited(format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)))
    }

    fn settled(&self) -> String {
        let events = self.clone.path().join(".git/meta-ci/ABCDEF1234/events");
        std::iter::repeat_with(|| {
            std::thread::sleep(SETTLE_POLL);
            std::fs::read_to_string(&events).unwrap_or_default()
        })
        .take(SETTLE_LENGTH)
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
    let waited = repo.session_command(CI_WAIT).0;
    let stop = repo.hook(STOP, COMMIT);
    let settled = repo.settled();
    let shown = format!("{waited}{stop}{}", repo.hook(PRE, COMMIT));
    assert!(waited.contains(JOB_FAILED) && stop.contains(BLOCK) && settled.contains(STEP_LISTED));
    assert!(shown.contains(STEP_LISTED) && shown.contains(CI_LOG_HINT));
    let log = repo.session_command(CI_LOG).0;
    assert!(log.contains(PANIC_BLOCK) && !log.contains(COMPILING));
}

#[test]
fn a_passing_run_reports_only_completion_and_lets_the_agent_stop() {
    let repo = Repo::new(SUCCESS);
    repo.hook("post-tool-use", COMMIT);
    assert!(repo.session_command(CI_WAIT).0.contains(PASSED) && !repo.hook(STOP, COMMIT).contains(BLOCK));
    assert_eq!(repo.settled().lines().filter(|line| line.contains(PASSED)).count(), repo.settled().lines().count());
}

#[test]
fn a_run_that_never_appears_times_out_and_says_so() {
    let repo = Repo::new(NONE);
    repo.hook("post-tool-use", COMMIT);
    assert!(repo.settled().contains(TIMED_OUT));
}
