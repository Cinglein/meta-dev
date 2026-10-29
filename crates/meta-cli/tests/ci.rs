#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use meta_golden::{Goldens, Replay};

#[allow(dead_code)]
#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/meta_command.rs"]
mod meta_command;
use fixture::{Fixture, Moment};
use meta_command::MetaCommand;

const REPLAY: Option<&str> = option_env!("CARGO_BIN_EXE_gh-replay");
const UNBUILT: &str = "gh-replay is built with feature `replay`; run the tests with --all-features";
const CONFIG: &str = "remote = \"origin\"\nprotected = [\"main\"]\n[ci]\npoll_millis = 2\n";
const PATIENT: &str =
    "expected_millis = 0\nbackoff_millis = 5\nbackoff_cap_millis = 20\ntimeout_millis = 400\ngh_millis = 350\n";
const EXPECTED: &str = ".git/meta-ci/expected";
const AT_ONCE: &str = "0";
const STALLED: &str = "expected_millis = 5\ntimeout_millis = 400\ngh_millis = 350\nstop_millis = 40\n";
const SETTLE_POLL: Duration = Duration::from_millis(2);
const SETTLE_LENGTH: usize = 1500;
const SESSION: &str = "ABCDEF12-34";
const SESSION_FILE: &str = "ABCDEF1234";
const FAILURE: Answer =
    Answer { checks: "01-failed-test-time", logs: &["02-failed-test-time-ci-log", "03-failed-test-time-gate-log"] };
const SUCCESS: Answer = Answer { checks: "04-passed", logs: &[] };
const NOT_WATCHED: &str = "CI not watched for";
const SLOW_REMOTE: [&str; 2] = ["remote.origin.uploadpack", "sleep 0.3; git-upload-pack"];
const SLOW: Duration = Duration::from_millis(500);
const OUT_OF_TIME: &str = "couldn't check CI in time; run `meta ci-wait`";
const PUSHED: &str = "feature";
const NEXT_COMMIT: &str = "next";
const LONG_AGO: Moment = Moment::after_epoch(1_000_000_000);
const POST: &str = "post-tool-use";
const COMMIT: &str = "git commit -m change";
const PRE: &str = "pre-tool-use";
const STOP: &str = "stop";
const BLOCK: &str = "\"decision\":\"block\"";
const BLOCK_REASON: &str = "\"reason\":\"meta-development: CI failed for your last push";
const JOB_FAILED: &str = "ci failed; run `meta ci-log`";
const STEP_LISTED: &str = ": ci / Run cargo xtask ci , ci-gate / Run echo '{; run `meta ci-log`";
const LOGGED: &str = "== ci / Run cargo xtask ci  ==\n";
const BLOCKING: &str = "budget over its blocking line:";
const COMPILING: &str = "Compiling";
const CI_WAIT: &str = "ci-wait";
const CI_LOG: &str = "ci-log";
const PASSED: &str = "passed";
const PASSED_EVENT: &str = ": passed after";
const EVENT_PREFIX: &str = "CI ";
const COMPLETIONS: [&str; 6] = [
    ": passed after",
    ": failed after",
    ": timed-out after",
    ": unavailable after",
    ": absent after",
    ": unchecked after",
];

#[path = "ci/pushes.rs"]
mod pushes;
#[path = "ci/review.rs"]
mod review;
#[path = "ci/waits.rs"]
mod waits;

#[derive(Clone, Copy)]
struct Answer {
    checks: &'static str,
    logs: &'static [&'static str],
}

impl Answer {
    const RECORDED: &'static str = "gh-commit-checks";
    const SHA: &'static str = "sha=";
    const SHA_LEN: usize = 40;
    const REPOSITORY: [(&'static str, &'static str); 4] = [
        ("[\"gh\",", "[\"gh-replay\","),
        ("owner=Cinglein", "owner=o"),
        ("name=meta-dev", "name=r"),
        (",\"--repo\",\"Cinglein/meta-dev\"", ""),
    ];
    const COPIES: [&'static str; 3] = ["a", "b", "c"];
    const FILES: [&'static str; 3] = ["out", "err", "status"];

    fn given(self, replies: &Path, head: &str) {
        std::fs::create_dir_all(replies).unwrap();
        let names = [self.checks].into_iter().chain(self.logs.iter().copied());
        names.for_each(|name| Self::COPIES.iter().for_each(|copy| Self::copied(replies, name, head, copy)));
    }

    fn copied(replies: &Path, name: &str, head: &str, copy: &str) {
        let recorded = Goldens::workspace().dir().join(Self::RECORDED).join(name);
        let argv = std::fs::read_to_string(recorded.with_extension("argv")).unwrap();
        let sha = argv.find(Self::SHA).map(|at| at + Self::SHA.len());
        let argv = sha.map_or(argv.clone(), |at| argv.replace(&argv[at..at + Self::SHA_LEN], head));
        let argv = Self::REPOSITORY.iter().fold(argv, |argv, (recorded, local)| argv.replace(recorded, local));
        let stem = replies.join(format!("{head}-{name}-{copy}"));
        std::fs::write(stem.with_extension("argv"), argv).unwrap();
        let copied = Self::FILES
            .iter()
            .try_for_each(|file| std::fs::copy(recorded.with_extension(file), stem.with_extension(file)).map(drop));
        copied.unwrap();
    }
}

struct Waited(String, ExitStatus);

struct StartFile;

impl StartFile {
    const ROOT: &'static str = "meta-sessions";
    const NAME: &'static str = "started";
    const WRITES: &'static str = "writes";
    const NANOS: &'static str = "000000000";

    fn plant(sessions: &Path, session: &str, seconds: &str) {
        let folder = sessions.join(Self::ROOT).join(session);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join(Self::NAME), [seconds, Self::NANOS].concat()).unwrap();
        std::fs::write(folder.join(Self::WRITES), "").unwrap();
    }
}

struct Planted;

impl Planted {
    const STATES: &'static str = "meta-ci/states";
    const LOCKS: &'static str = "meta-ci/locks";

    fn locked(dir: &Path) -> std::fs::Permissions {
        let open = std::fs::metadata(dir).unwrap().permissions();
        let mut locked = open.clone();
        locked.set_readonly(true);
        std::fs::set_permissions(dir, locked).unwrap();
        open
    }

    fn twin(repo: &Repo) -> std::process::Child {
        let head = repo.git().to_string();
        repo.command(&["ci-watch", SESSION, &head]).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap()
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
    origin: PathBuf,
    clone: tempfile::TempDir,
    sessions: tempfile::TempDir,
    answer: Answer,
}

impl Repo {
    const ORIGIN: &'static str = "o/r";
    const REPLIES: &'static str = "replies";
    const TRAIL: &'static str = "replies-trail";

    fn new(answer: Answer) -> Self {
        Self::limited(answer, PATIENT)
    }

    pub fn limited(answer: Answer, limits: &str) -> Self {
        let sessions = tempfile::tempdir().unwrap();
        let origin = sessions.path().join(Self::ORIGIN);
        let clone = tempfile::tempdir().unwrap();
        Fixture::bare(&origin, PUSHED);
        Fixture::working(clone.path(), &origin, PUSHED).committed_at("change", &LONG_AGO);
        StartFile::plant(sessions.path(), SESSION_FILE, "0");
        std::fs::create_dir(sessions.path().join(Self::TRAIL)).unwrap();
        let repo = Self { origin, clone, sessions, answer };
        repo.configured(limits);
        repo.answered();
        repo
    }

    pub fn answered(&self) {
        self.answer.given(&self.sessions.path().join(Self::REPLIES), &self.git().to_string());
    }

    fn git(&self) -> Fixture {
        Fixture::opened(self.clone.path())
    }

    pub fn configured(&self, limits: &str) {
        let program = format!("program = {:?}\n", REPLAY.filter(|built| Path::new(built).exists()).expect(UNBUILT));
        std::fs::write(self.clone.path().join("meta.toml"), format!("{CONFIG}{program}{limits}")).unwrap();
    }

    fn hook(&self, event: &str, command: &str) -> String {
        Session { repo: self, cwd: self.clone.path(), id: SESSION }.hook(event, command)
    }

    pub fn passing(self) -> Self {
        let repo = Self { answer: SUCCESS, ..self };
        repo.answered();
        repo
    }

    pub fn command(&self, words: &[&str]) -> Command {
        let mut command = MetaCommand::tagged(words);
        command.current_dir(self.clone.path()).env("TMPDIR", self.sessions.path());
        let replies = self.sessions.path().join(Self::REPLIES);
        command.env(Replay::RECORDINGS, replies).env(Replay::TRAIL, self.sessions.path().join(Self::TRAIL));
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
        let events = self.clone.path().join(".git/meta-ci/ABCDEF1234/events");
        let lock = self.sessions.path().join(Planted::LOCKS).join(self.git().to_string());
        let pauses = std::iter::successors(Some(Duration::ZERO), |_| Some(SETTLE_POLL)).take(SETTLE_LENGTH);
        pauses
            .map(std::thread::sleep)
            .map(|()| std::fs::read_to_string(&events).unwrap_or_default())
            .find(|text| {
                let new = text.strip_prefix(seen).unwrap_or_default();
                COMPLETIONS.iter().any(|completion| new.contains(completion))
                    && std::fs::File::open(&lock).map_or(true, |watcher| watcher.try_lock().is_ok())
            })
            .unwrap_or_default()
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
    Planted::twin(&repo).wait().unwrap();
    let posted = repo.hook(POST, COMMIT);
    let waited = repo.meta(&[CI_WAIT]);
    let stop = repo.hook(STOP, COMMIT);
    let shown = format!("{posted}{}{stop}{}", waited.0, repo.hook(PRE, COMMIT));
    let told = [&posted, &waited.0].iter().filter(|told| told.contains(JOB_FAILED)).count();
    assert!(told == 1 && !waited.1.success() && stop.contains(BLOCK_REASON) && shown.contains(STEP_LISTED));
    repo.git().committed(NEXT_COMMIT);
    let log = repo.meta(&[CI_LOG]).0;
    repo.git().committed(NEXT_COMMIT);
    let repo = repo.passing();
    std::fs::write(repo.clone.path().join(EXPECTED), AT_ONCE).unwrap();
    Planted::twin(&repo).wait().unwrap();
    repo.hook(POST, COMMIT);
    let fixed = repo.hook(STOP, COMMIT);
    assert!(
        log.contains(LOGGED) && log.contains(BLOCKING) && !log.contains(COMPILING) && !fixed.contains(BLOCK),
        "{log}"
    );
}

#[test]
fn a_passing_push_lets_the_agent_stop_and_a_commit_whose_state_cannot_be_written_is_not_watched() {
    let repo = Repo::new(SUCCESS);
    let watched = String::from_utf8(Planted::twin(&repo).wait_with_output().unwrap().stdout).unwrap();
    repo.hook(POST, COMMIT);
    let waited = repo.meta(&[CI_WAIT]);
    assert!(watched.contains(PASSED) && waited.1.success() && !repo.hook(STOP, COMMIT).contains(BLOCK));
    assert!(repo.settled().lines().all(|line| line.starts_with(EVENT_PREFIX) && line.contains(PASSED_EVENT)));
    let next = repo.git().committed(NEXT_COMMIT);
    std::fs::create_dir_all(repo.sessions.path().join(Planted::STATES).join(next)).unwrap();
    repo.hook(POST, COMMIT);
    let unwatched = repo.hook(STOP, COMMIT);
    repo.configured(STALLED);
    let [key, value] = SLOW_REMOTE;
    repo.git().configured(key, value);
    let started = Instant::now();
    let slow = repo.hook(STOP, COMMIT);
    assert!(unwatched.contains(NOT_WATCHED) && slow.contains(OUT_OF_TIME) && started.elapsed() < SLOW);
}
