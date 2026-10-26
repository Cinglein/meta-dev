#![allow(clippy::unwrap_used)]

use std::fmt;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::time::Duration;

const META: &str = env!("CARGO_BIN_EXE_meta");
const CONFIG: &str =
    "remote = \"origin\"\nprotected = [\"main\"]\n[ci]\nprogram = \"gh\"\npoll_millis = 10\nappear_millis = 60\n";
const PATIENT: &str = "timeout_millis = 120\n";
const FAKE_GH: &str = r#"#!/bin/sh
count="$FAKE_GH_DIR/count"
n=$(cat "$count" 2>/dev/null || echo 0)
s="$FAKE_GH_SCENARIO"
case "$1 $2" in
"run list")
  n=$((n + 1)); echo "$n" > "$count"
  if [ "$s" = broken ] && [ "$n" -lt 3 ]; then echo 'HTTP 502: Bad Gateway' >&2; exit 1
  elif [ "$s" = none ] || [ "$4" != "$(git rev-parse HEAD)" ]; then echo '[]'
  elif [ "$n" -lt 2 ] || { [ "$s" = late ] && [ "$n" -lt 8 ]; }; then echo '[]'
  elif [ "$s" = late ] || [ "$n" -lt 4 ]; then echo '[{"databaseId":7,"status":"in_progress","conclusion":""}]'
  elif [ "$s" = success ] || [ "$s" = broken ]; then echo '[{"databaseId":7,"status":"completed","conclusion":"success"}]'
  elif [ "$s" = mixed ]; then echo '[{"databaseId":7,"status":"completed","conclusion":"failure"},{"databaseId":8,"status":"completed","conclusion":"failure"}]'
  else echo '[{"databaseId":7,"status":"completed","conclusion":"failure"}]'; fi ;;
"run view")
  if [ "$s" = broken ]; then echo 'HTTP 401: Bad credentials' >&2; exit 1
  elif [ "$3 $4" = "--job 11" ]; then cat "$FAKE_GH_DIR/bin/failed.log"
  elif [ "$3" = 8 ]; then echo '{"jobs":[{"databaseId":13,"name":"lint","conclusion":"failure","steps":[{"name":"clippy","conclusion":"failure"}]}]}'
  elif [ "$3" != 7 ]; then echo '{"jobs":[]}'
  elif [ "$s" = failure ]; then echo '{"jobs":[{"databaseId":11,"name":"ci","conclusion":"failure","steps":[{"name":"Checkout","conclusion":"success"},{"name":"cargo xtask ci","conclusion":"failure"}]},{"databaseId":12,"name":"docs","conclusion":""}]}'
  elif [ "$s" = unstarted ] || [ "$s" = mixed ]; then echo '{"jobs":[{"databaseId":11,"name":"ci","conclusion":"failure","steps":[]}]}'
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
const LATE: &str = "late";
const BROKEN: &str = "broken";
const MIXED: &str = "mixed";
const UNSTARTED: &str = "unstarted";
const POST: &str = "post-tool-use";
const COMMIT: &str = "git commit -m change";
const PRE: &str = "pre-tool-use";
const STOP: &str = "stop";
const BLOCK: &str = "\"decision\":\"block\"";
const BLOCK_REASON: &str = "\"reason\":\"meta-development: CI failed for your last push";
const JOB_FAILED: &str = "ci failed; run `meta ci-log`";
const CI_LOG_HINT: &str = "run `meta ci-log`";
const STEP_LISTED: &str = "s: ci / cargo xtask ci; run `meta ci-log`";
const PANIC_BLOCK: &str =
    "panicked at crates/meta-state/src/state/tests.rs:42:5:\nassertion `left == right` failed\n  left: None";
const COMPILING: &str = "Compiling";
const CI_WAIT: &str = "ci-wait";
const CI_LOG: &str = "ci-log";
const PASSED: &str = "passed after";
const PASSED_EVENT: &str = ": passed after";
const EVENT_PREFIX: &str = "CI ";
const TIMED_OUT: &str = "timed-out after";
const UNAVAILABLE: &str = ": jobs never started; give the local gate result instead";
const ABSENT: &str = ": no run for this commit; no CI for this push";
const NO_RUN_YET: &str = "no run yet; Stop passes until one appears";
const RUN_APPEARED: &str = "run appeared";
const LIST_ERROR: &str = "gh failed: HTTP 502: Bad Gateway";
const VIEW_ERROR: &str = "gh failed: HTTP 401: Bad credentials";
const FAILED_EVENT: &str = ": failed after";
const COMPLETIONS: [&str; 5] =
    [": passed after", ": failed after", ": timed-out after", ": unavailable after", ": absent after"];

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

struct Waited(String, ExitStatus);

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
        std::fs::write(clone.path().join("meta.toml"), format!("{CONFIG}{PATIENT}")).unwrap();
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
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        Waited(text, output.status)
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
    repo.hook(POST, COMMIT);
    let waited = repo.session_command(CI_WAIT);
    let stop = repo.hook(STOP, COMMIT);
    let settled = repo.settled();
    let shown = format!("{}{stop}{}", waited.0, repo.hook(PRE, COMMIT));
    assert!(waited.0.contains(JOB_FAILED) && !waited.1.success() && stop.contains(BLOCK_REASON));
    assert!(settled.contains(STEP_LISTED) && shown.contains(STEP_LISTED) && shown.contains(CI_LOG_HINT));
    let log = repo.session_command(CI_LOG).0;
    assert!(log.contains(PANIC_BLOCK) && !log.contains(COMPILING));
}

#[test]
fn passing_or_unstarted_runs_let_the_agent_stop() {
    let repo = Repo::new(SUCCESS);
    repo.hook(POST, COMMIT);
    let waited = repo.session_command(CI_WAIT);
    assert!(waited.0.contains(PASSED) && waited.1.success() && !repo.hook(STOP, COMMIT).contains(BLOCK));
    assert!(repo.settled().lines().all(|line| line.starts_with(EVENT_PREFIX) && line.contains(PASSED_EVENT)));
    let unstarted = Repo::new(UNSTARTED);
    unstarted.hook(POST, COMMIT);
    let settled = unstarted.settled();
    let wait = unstarted.session_command(CI_WAIT).1;
    assert!(settled.contains(UNAVAILABLE) && !unstarted.hook(STOP, COMMIT).contains(BLOCK) && !wait.success());
}

#[test]
fn only_a_run_that_never_appears_waives_ci_and_gh_errors_never_do() {
    let repos = [NONE, LATE, BROKEN, MIXED].map(Repo::new);
    let _: Vec<String> = repos.iter().map(|repo| repo.hook(POST, COMMIT)).collect();
    let [absent, late, broken, mixed] = repos.each_ref().map(Repo::settled);
    let [none, ..] = &repos;
    assert!(absent.contains(ABSENT) && !none.hook(STOP, COMMIT).contains(BLOCK));
    assert!(late.contains(NO_RUN_YET) && late.contains(RUN_APPEARED) && late.contains(TIMED_OUT));
    assert!(broken.contains(LIST_ERROR) && broken.contains(VIEW_ERROR) && mixed.contains(FAILED_EVENT));
}
