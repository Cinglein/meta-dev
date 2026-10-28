#![allow(clippy::unwrap_used)]

use meta_door::kinds::Permissions;
use std::fmt;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::time::Duration;

use meta_door::{Change, Door, Entry, MetaStore, Opening};

const META: &str = env!("CARGO_BIN_EXE_meta");
const CONFIG: &str = "remote = \"origin\"\nprotected = [\"main\"]\n[ci]\nprogram = \"gh\"\npoll_millis = 2\n";
const PATIENT: &str = "appear_millis = 2000\n";
const STALLED: &str = "appear_millis = 2000\nstop_millis = 40\n";
const QUICK: &str = "appear_millis = 20\ntimeout_millis = 50\ngh_millis = 20\n";
const DEADLINE: &str =
    "expected_millis = 30\nbackoff_millis = 60\nbackoff_cap_millis = 60\ntimeout_millis = 100\ngh_millis = 20\n";
const FAKE_GH: &str = r#"#!/bin/sh
d="$FAKE_GH_DIR"; s="$FAKE_GH_SCENARIO"
read -r now 2>/dev/null < "$d/clock"; echo "$now" >> "$d/calls"
case "$s" in hang) exec sleep 5 ;; frozen) echo 0 > "$d/clock"; exec sleep 5 ;; esac
[ "$3 $4" = "--job 11" ] && exec cat "$d/bin/failed.log"
read -r n 2>/dev/null < "$d/count" || n=0; n=$((n + 1)); echo "$n" > "$d/count"
read -r head < "$d/bin/head"
events="$d/.git/meta-ci/ABCDEF1234/events"
state=IN_PROGRESS; result=null; jobs='{"databaseId":11,"name":"ci","conclusion":"SUCCESS"}'
case "$s" in failure|broken) jobs='{"databaseId":11,"name":"ci","conclusion":"FAILURE","steps":{"nodes":[{"name":"Checkout","conclusion":"SUCCESS"},{"name":"cargo xtask ci","conclusion":"FAILURE"}]}},{"databaseId":12,"name":"docs"}' ;;
unstarted|mixed) jobs='{"databaseId":11,"name":"ci","conclusion":"FAILURE"}' ;; esac
case "$s$*" in failure*filterBy*|broken*filterBy*) ;; failure*|broken*) pad=; i=0; while [ "$i" -lt 50 ]; do pad="$pad{\"databaseId\":$((i + 100)),\"name\":\"job$i\",\"conclusion\":\"SUCCESS\"},"; i=$((i + 1)); done; jobs="${pad%,}" ;; esac
case "$s" in late|broken|slow) ;; last) [ "$n" -ge 1 ] && state=COMPLETED result='"SUCCESS"' ;; success) [ "$n" -ge 4 ] && state=COMPLETED result='"SUCCESS"' ;; *) [ "$n" -ge 4 ] && state=COMPLETED result='"FAILURE"' ;; esac
runs="{\"status\":\"$state\",\"conclusion\":$result,\"workflowRun\":{},\"checkRuns\":{\"nodes\":[$jobs]}}"
[ "$s" = mixed ] && [ "$state" = COMPLETED ] && runs="$runs,{\"status\":\"COMPLETED\",\"conclusion\":\"FAILURE\",\"workflowRun\":{},\"checkRuns\":{\"nodes\":[{\"databaseId\":13,\"name\":\"lint\",\"conclusion\":\"FAILURE\",\"steps\":{\"nodes\":[{\"name\":\"clippy\",\"conclusion\":\"FAILURE\"}]}}]}}"
case "$*" in *appId:15368*) ;; *) foreign=; i=0; while [ "$i" -lt 20 ]; do foreign="$foreign{\"status\":\"COMPLETED\",\"conclusion\":\"SUCCESS\"},"; i=$((i + 1)); done; runs="${foreign%,}" ;; esac
if [ "$s" = broken ] && [ "$n" -lt 3 ]; then echo 'HTTP 502: Bad Gateway' >&2; exit 1
elif [ "$s" = broken ] && [ "$n" -gt 5 ]; then echo 'HTTP 401: Bad credentials' >&2; exit 1
elif [ "$s" = none ] || [ "$4" != "sha=$head" ]; then runs=
elif [ "$s" = late ] && { ! grep -q "no run yet" "$events" || grep -q "run appeared" "$events"; }; then runs=
fi
past='{"checkSuites":{"nodes":[{"status":"COMPLETED","workflowRun":{},"createdAt":"2026-10-27T10:00:00Z","updatedAt":"2026-10-27T10:04:00Z"}]}}'
echo "{\"checkSuites\":{\"nodes\":[$runs]},\"history\":{\"nodes\":[$past]}}"
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
const SETTLE_POLL: Duration = Duration::from_millis(2);
const SETTLE_LENGTH: usize = 3500;
const SESSION: &str = "ABCDEF12-34";
const SESSION_FILE: &str = "ABCDEF1234";
const FAILURE: &str = "failure";
const SUCCESS: &str = "success";
const NONE: &str = "none";
const LATE: &str = "late";
const BROKEN: &str = "broken";
const MIXED: &str = "mixed";
const HANG: &str = "hang";
const LAST: &str = "last";
const FROZEN: &str = "frozen";
const INJECTED: &str = " (injected clock)";
const FOURTH_POLL: &str = ": failed after 510s";
const NOT_WATCHED: &str = "CI not watched for";
const PLAIN_PUSH: &str = "git push";
const PUSHED: &str = "feature";
const NEXT_COMMIT: [&str; 5] = ["commit", "-q", "--allow-empty", "-m", "next"];
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
const UNAVAILABLE: &str = ": jobs never started; push a new commit to retry, then run `meta ci-wait`";
const ABSENT: &str = ": no run for this commit; no CI for this push";
const NO_RUN_YET: &str = "no run yet; Stop stays blocked until one appears or the wait is recorded";
const ABSENT_STOP: &str = "no CI run for your last push: push a new commit to start one, then run `meta ci-wait`; or open or claim a task (`meta open <text> --division <path>`, `meta claim <task>`), then run `meta wait <task> --on ci` here";
const TIMED_OUT_STOP: &str = "CI for your last push timed out: push a new commit to rerun it, then run `meta ci-wait`";
const UNSTARTED_STOP: &str =
    "CI jobs for your last push never started: push a new commit to retry, then run `meta ci-wait`";
const RUN_APPEARED: &str = "run appeared";
const LIST_ERROR: &str = "gh failed: HTTP 502: Bad Gateway";
const VIEW_ERROR: &str = "gh failed: HTTP 401: Bad credentials";
const FAILED_EVENT: &str = ": failed after";
const COMPLETIONS: [&str; 6] = [
    ": passed after",
    ": failed after",
    ": timed-out after",
    ": unavailable after",
    ": absent after",
    ": unchecked after",
];

#[path = "support/seeded_log.rs"]
mod seeded_log;
#[path = "support/skeleton.rs"]
mod skeleton;
use skeleton::Skeleton;
#[path = "ci/merges.rs"]
mod merges;
#[path = "ci/pushes.rs"]
mod pushes;
#[path = "ci/review.rs"]
mod review;
#[path = "ci/sessions.rs"]
mod sessions;
#[path = "ci/waits.rs"]
mod waits;

struct Git;

impl Git {
    const IDENTITY: [&'static str; 8] = [
        "-c",
        "user.name=t",
        "-c",
        "user.email=t@meta.invalid",
        "-c",
        "init.defaultBranch=main",
        "-c",
        "maintenance.auto=false",
    ];

    const LONG_AGO: &'static str = "@1000000000 +0000";

    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }

    fn run_at(dir: &Path, words: &[&str], date: &str) -> Output {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(Self::IDENTITY)
            .args(words)
            .env("GIT_COMMITTER_DATE", date)
            .output()
            .unwrap()
    }

    fn head(dir: &Path) -> String {
        Door::laid_head(dir).unwrap_or_else(|_| {
            String::from_utf8(Self::run(dir, &["rev-parse", "HEAD"]).stdout).unwrap().trim().to_owned()
        })
    }

    fn pushed(repo: &Repo, branch: &str) {
        let tip = format!("{}\n", Self::head(repo.clone.path()));
        Door::laid_file(repo.origin.path(), &Path::new("refs/heads").join(branch), &tip).unwrap();
        let tracking = Path::new("refs/remotes/origin").join(branch);
        Door::laid_file(&repo.clone.path().join(".git"), &tracking, &tip).unwrap();
    }
}

struct Origin;

impl Origin {
    const ALTERNATES: &'static str = "objects/info/alternates";

    fn linked(origin: &Path, clone: &Path) {
        let objects = format!("{}\n", clone.join(".git/objects").display());
        Door::laid_file(origin, Path::new(Self::ALTERNATES), &objects).unwrap();
    }
}

struct Waited(String, ExitStatus);

struct Stale;

impl Stale {
    fn plant(repo: &Repo) {
        let head = Git::head(repo.clone.path());
        Door::changed(repo.sessions.path().join(Planted::STATES).join(head.trim()), Change::DirAll).unwrap();
    }
}

struct Started;

impl Started {
    fn building(scenario: &'static str, limits: &'static str) -> std::thread::JoinHandle<Repo> {
        std::thread::spawn(move || {
            let repo = Repo::limited(scenario, limits);
            (scenario == NONE).then(|| Calls::plant(&repo));
            repo.hook(POST, COMMIT);
            repo
        })
    }
}

struct Calls;

impl Calls {
    const MAX_LEN: usize = 10;
    const EXPECTED: &'static str = "meta-ci/expected";
    const PLANTED: &'static str = "60000";
    const LEARNED: &'static str = "240000";

    fn plant(repo: &Repo) {
        Self::expect(repo, Self::PLANTED);
    }

    fn expect(repo: &Repo, millis: &str) {
        let expected = repo.store().entry(Self::EXPECTED);
        Door::changed(expected.map(|file| file.with_file_name("")), Change::DirAll).unwrap();
        Door::write(expected, millis).unwrap();
    }

    fn within_budget(repo: &Repo) -> bool {
        let log = Door::read(repo.clone.path().join("calls")).unwrap_or_default();
        let polls: std::collections::BTreeSet<&str> = log.lines().collect();
        let learned = Door::read(repo.store().entry(Self::EXPECTED)).unwrap_or_default();
        (1..=Self::MAX_LEN).contains(&polls.len())
            && polls.len() == log.lines().count()
            && [log.lines().next(), Some(learned.as_str())] == [Some(Self::PLANTED), Some(Self::LEARNED)]
    }
}

struct StartFile;

impl StartFile {
    const ROOT: &'static str = "meta-sessions";
    const NAME: &'static str = "started";
    const WRITES: &'static str = "writes";
    const NANOS: &'static str = "000000000";

    fn second_of(sessions: &Path, session: &str) -> String {
        let nanos = Door::read(sessions.join(Self::ROOT).join(session).join(Self::NAME)).unwrap();
        nanos.split_at(nanos.len() - Self::NANOS.len()).0.to_owned()
    }

    fn plant(sessions: &Path, session: &str, seconds: &str) {
        let folder = sessions.join(Self::ROOT).join(session);
        Door::changed(&folder, Change::DirAll).unwrap();
        Door::write(folder.join(Self::NAME), [seconds, Self::NANOS].concat()).unwrap();
        Door::write(folder.join(Self::WRITES), "").unwrap();
    }
}

struct FailingHead;

impl FailingHead {
    fn copy_from(repo: &Repo, dir: &Path) {
        Door::write(repo.clone.path().join("bin/head"), format!("{}\n", Git::head(dir))).unwrap();
    }
}

struct Planted;

impl Planted {
    const STATES: &'static str = "meta-ci/states";
    const LOCKS: &'static str = "meta-ci/locks";
    const HOOKS: &'static str = "bin/hooks";
    const PRE_RECEIVE: &'static str = "bin/hooks/pre-receive";
    const REFUSE: &'static str = "#!/bin/sh\nexit 1\n";
    const RUNNING: &'static str = "running";

    fn settled(repo: &Repo, ci: &str) {
        let head = Git::head(repo.clone.path());
        let states = repo.sessions.path().join(Self::STATES);
        Door::changed(&states, Change::DirAll).unwrap();
        Door::write(states.join(head.trim()), format!("{} {ci}", head.trim())).unwrap();
    }

    fn refusal(repo: &Repo) {
        let hooks = repo.clone.path().join(Self::HOOKS);
        Door::changed(&hooks, Change::DirAll).unwrap();
        let hook = repo.clone.path().join(Self::PRE_RECEIVE);
        Door::write(&hook, Self::REFUSE).unwrap();
        let executable = Door::described(Door::canonical(SHELL).unwrap()).unwrap().permissions();
        Door::changed(hook, Change::Permissions(executable)).unwrap();
        let hooks = hooks.display().to_string();
        assert!(Git::run(repo.origin.path(), &["config", "core.hooksPath", &hooks]).status.success());
    }

    fn locked(dir: impl Into<Entry>) -> Permissions {
        let dir = dir.into();
        let open = Door::described(dir.clone()).unwrap().permissions();
        let mut locked = open.clone();
        locked.set_readonly(true);
        Door::changed(dir, Change::Permissions(locked)).unwrap();
        open
    }

    fn twin(repo: &Repo) -> std::process::Child {
        let head = Door::read(repo.clone.path().join("bin/head")).unwrap();
        repo.command(&["ci-watch", SESSION, head.trim()]).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap()
    }
}

struct Session<'repo> {
    repo: &'repo Repo,
    cwd: &'repo Path,
    id: &'repo str,
}

impl Session<'_> {
    fn hook(&self, event: &str, command: &str) -> String {
        let input = format!(
            r#"{{"session_id":{},"cwd":{},"tool_input":{{"command":{}}}}}"#,
            serde_json::to_string(self.id).unwrap(),
            serde_json::to_string(self.cwd).unwrap(),
            serde_json::to_string(command).unwrap()
        );
        let mut child =
            self.repo.command(&["hook", event]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        String::from_utf8_lossy(&child.wait_with_output().unwrap().stdout).into_owned()
    }
}

struct Repo {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
    sessions: tempfile::TempDir,
    scenario: &'static str,
}

impl Repo {
    const EVENTS: &'static str = "meta-ci/ABCDEF1234/events";

    fn new(scenario: &'static str) -> Self {
        Self::limited(scenario, PATIENT)
    }

    pub fn limited(scenario: &'static str, limits: &str) -> Self {
        let origin = tempfile::tempdir().unwrap();
        let clone = tempfile::tempdir().unwrap();
        Skeleton::laid(origin.path(), clone.path(), "feature");
        Origin::linked(origin.path(), clone.path());
        Door::write(clone.path().join("meta.toml"), format!("{CONFIG}{limits}")).unwrap();
        Door::changed(clone.path().join("bin"), Change::Dir).unwrap();
        Door::write(clone.path().join("bin/gh"), FAKE_GH).unwrap();
        Door::write(clone.path().join("bin/failed.log"), FAILED_LOG).unwrap();
        let executable = Door::described(Door::canonical(SHELL).unwrap()).unwrap().permissions();
        Door::changed(clone.path().join("bin/gh"), Change::Permissions(executable)).unwrap();
        Git::run_at(clone.path(), &["commit", "-q", "--allow-empty", "-m", "change"], Git::LONG_AGO);
        let sessions = tempfile::tempdir().unwrap();
        StartFile::plant(sessions.path(), SESSION_FILE, "0");
        let repo = Self { origin, clone, sessions, scenario };
        FailingHead::copy_from(&repo, repo.clone.path());
        repo
    }

    fn hook(&self, event: &str, command: &str) -> String {
        Session { repo: self, cwd: self.clone.path(), id: SESSION }.hook(event, command)
    }

    fn session_command(&self, command: &str) -> Waited {
        self.meta(&[command])
    }

    pub fn passing(self) -> Self {
        FailingHead::copy_from(&self, self.clone.path());
        Self { scenario: SUCCESS, ..self }
    }

    pub fn command(&self, words: &[&str]) -> Command {
        let path = format!("{}:{}", self.clone.path().join("bin").display(), std::env::var("PATH").unwrap_or_default());
        let mut command = Command::new(META);
        command.args(words).current_dir(self.clone.path()).env("PATH", path).env("TMPDIR", self.sessions.path());
        command.env("FAKE_GH_DIR", self.clone.path()).env("FAKE_GH_SCENARIO", self.scenario);
        command.envs(
            [HANG, LAST]
                .iter()
                .all(|real| *real != self.scenario)
                .then(|| ("META_TEST_CLOCK", self.clone.path().join("clock"))),
        );
        command
    }

    pub fn meta(&self, words: &[&str]) -> Waited {
        let output = self.command(words).env("META_SESSION", SESSION).output().unwrap();
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        Waited(text, output.status)
    }

    fn settled(&self) -> String {
        self.settled_past("")
    }

    pub fn settled_past(&self, seen: &str) -> String {
        let events = self.store().entry(Self::EVENTS);
        let lock = self.sessions.path().join(Planted::LOCKS).join(Git::head(self.clone.path()));
        let pauses = std::iter::successors(Some(Duration::ZERO), |_| Some(SETTLE_POLL)).take(SETTLE_LENGTH);
        pauses
            .map(std::thread::sleep)
            .map(|()| Door::read(events.clone()).unwrap_or_default())
            .find(|text| {
                let new = text.strip_prefix(seen).unwrap_or_default();
                COMPLETIONS.iter().any(|completion| new.contains(completion))
                    && Door::opened(&lock, Opening::Read).map_or(true, |watcher| watcher.try_lock().is_ok())
            })
            .unwrap_or_default()
    }

    pub fn store(&self) -> MetaStore {
        MetaStore::located(self.clone.path()).unwrap()
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.clone.path().display())
    }
}

#[test]
fn ci_wait_returns_at_the_first_failure_and_stop_stays_blocked_until_a_later_push() {
    let repo = Repo::new(FAILURE);
    let posted = repo.hook(POST, COMMIT);
    let waited = repo.session_command(CI_WAIT);
    let stop = repo.hook(STOP, COMMIT);
    let settled = repo.settled();
    let shown = format!("{posted}{}{stop}{}", waited.0, repo.hook(PRE, COMMIT));
    let told = [&posted, &waited.0].iter().filter(|told| told.contains(JOB_FAILED)).count();
    assert!(told == 1 && !waited.1.success() && stop.contains(BLOCK_REASON));
    assert!(settled.contains(STEP_LISTED) && shown.contains(STEP_LISTED) && shown.contains(CI_LOG_HINT));
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    let log = repo.session_command(CI_LOG).0;
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    let repo = repo.passing();
    repo.hook(POST, COMMIT);
    repo.session_command(CI_WAIT);
    let fixed = repo.hook(STOP, COMMIT);
    assert!(log.contains(PANIC_BLOCK) && !log.contains(COMPILING) && !fixed.contains(BLOCK));
}

#[test]
fn passing_runs_let_the_agent_stop_and_unstarted_ones_do_not() {
    let unstarted = Started::building(UNSTARTED, PATIENT);
    let repo = Repo::new(SUCCESS);
    Git::pushed(&repo, PUSHED);
    repo.hook(POST, PLAIN_PUSH);
    let waited = repo.session_command(CI_WAIT);
    assert!(waited.0.contains(PASSED) && waited.1.success() && !repo.hook(STOP, COMMIT).contains(BLOCK));
    assert!(repo.settled().lines().all(|line| line.starts_with(EVENT_PREFIX) && line.contains(PASSED_EVENT)));
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    Stale::plant(&repo);
    Git::pushed(&repo, PUSHED);
    repo.hook(POST, PLAIN_PUSH);
    let unstarted = unstarted.join().unwrap();
    let settled = unstarted.settled();
    let wait = unstarted.session_command(CI_WAIT).1;
    let unwatched = repo.hook(STOP, COMMIT);
    assert!(
        settled.contains(UNAVAILABLE)
            && unstarted.hook(STOP, COMMIT).contains(UNSTARTED_STOP)
            && !wait.success()
            && unwatched.contains(NOT_WATCHED)
    );
}

#[test]
fn absent_and_timed_out_runs_block_stop_and_gh_errors_fail_them() {
    let repos = [(NONE, PATIENT), (LATE, PATIENT), (BROKEN, PATIENT), (MIXED, PATIENT), (HANG, QUICK), (FROZEN, QUICK)]
        .map(|(scenario, limits)| Started::building(scenario, limits))
        .map(|started| started.join().unwrap());
    let [absent, late, broken, mixed, hung, frozen] = repos.each_ref().map(Repo::settled);
    let [none, appeared, _, _, hanging, _] = &repos;
    let budgeted = Calls::within_budget(none);
    let waited = none.session_command(CI_WAIT);
    let [appeared, hanging_stop] = [appeared, hanging].map(|repo| repo.hook(STOP, COMMIT));
    assert!(
        absent.contains(ABSENT)
            && budgeted
            && absent.split_once(NO_RUN_YET).is_some_and(|(_, after)| !after.contains(NO_RUN_YET))
            && none.hook(STOP, COMMIT).contains(&format!("{ABSENT_STOP}{INJECTED}"))
            && !waited.1.success()
            && waited.0.contains(&format!("CI absent{INJECTED}"))
    );
    assert!(
        [&late, &hung, &frozen].iter().all(|settled| settled.contains(TIMED_OUT))
            && [&appeared, &hanging_stop].iter().all(|stop| stop.contains(TIMED_OUT_STOP))
            && late.contains(NO_RUN_YET)
            && late.contains(RUN_APPEARED)
    );
    assert!(
        broken.contains(LIST_ERROR)
            && broken.contains(VIEW_ERROR)
            && broken.contains(FAILED_EVENT)
            && mixed.contains(FOURTH_POLL)
            && [broken, frozen].iter().all(|settled| settled.lines().all(|line| line.contains(INJECTED)))
            && ![hung, hanging_stop, hanging.session_command(CI_WAIT).0].iter().any(|real| real.contains(INJECTED))
    );
}
