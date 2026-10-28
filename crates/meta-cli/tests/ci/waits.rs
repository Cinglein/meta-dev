use std::collections::BTreeSet;

use super::seeded_log::{SeededLog, TASK};
use super::{BLOCK, COMMIT, Git, NEXT_COMMIT, POST, Planted, Repo, SHELL, STOP, SUCCESS, TIMED_OUT_STOP};

const TIMED_OUT: &str = "timed-out";
const SLOW: &str = "slow";
const CALLS: &str = "calls";
const CALL_LOG_MAX_LEN: usize = 10;
const STATE_ROOT: &str = ".git/meta-ci";
const EXPECTED: &str = "expected";
const PLANTED: &str = "60000";
const LEARNED: &str = "240000";
const DETACH: [&str; 3] = ["checkout", "-q", "--detach"];
const PUSH_HEAD: [&str; 4] = ["push", "-q", "origin", "HEAD:refs/heads/pushed"];
const WAIT_ON_CI: [&str; 4] = ["wait", TASK, "--on", "ci"];
const TASKLESS: &str = "then run `meta wait <task> --on ci` here";
const POST_AND_STOP_CALLS: [&str; 9] =
    ["cat-file", "log", "log", "ls-remote", "merge-base", "rev-list", "rev-list", "rev-parse", "rev-parse"];

struct Unresolved;

impl Unresolved {
    fn pushed(repo: &Repo) -> String {
        Git::run(repo.clone.path(), &PUSH_HEAD);
        Planted::settled(repo, TIMED_OUT);
        repo.hook(STOP, COMMIT)
    }
}

struct Counted;

impl Counted {
    const GIT: &'static str =
        "#!/bin/sh\necho \"$3\" >> \"$FAKE_GH_DIR/git-calls\"\nPATH=\"${PATH#*:}\" exec git \"$@\"\n";
    const WRAPPER: &'static str = "bin/git";
    const CALLS: &'static str = "git-calls";

    fn install(repo: &Repo) {
        let wrapper = repo.clone.path().join(Self::WRAPPER);
        std::fs::write(&wrapper, Self::GIT).unwrap();
        std::fs::set_permissions(&wrapper, std::fs::metadata(SHELL).unwrap().permissions()).unwrap();
    }

    fn calls(repo: &Repo) -> Vec<String> {
        let text = std::fs::read_to_string(repo.clone.path().join(Self::CALLS)).unwrap_or_default();
        let mut calls: Vec<String> = text.lines().map(str::to_owned).collect();
        calls.sort_unstable();
        calls
    }
}

#[test]
fn on_a_detached_head_a_wait_covers_only_its_head_and_a_typical_stop_lists_the_remote_once() {
    let repo = Repo::new(SUCCESS);
    SeededLog::CLAIMED.plant(repo.origin.path());
    Git::run(repo.clone.path(), &DETACH);
    let unrecorded = Unresolved::pushed(&repo);
    repo.meta(&WAIT_ON_CI);
    let recorded = repo.hook(STOP, COMMIT);
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    Counted::install(&repo);
    repo.hook(POST, COMMIT);
    let moved_on = Unresolved::pushed(&repo);
    assert!(unrecorded.contains(TIMED_OUT_STOP) && unrecorded.contains(TASKLESS));
    assert!(serde_json::from_str::<serde_json::Value>(&unrecorded).is_ok() && !recorded.contains(BLOCK));
    assert!(moved_on.contains(TASKLESS) && Counted::calls(&repo) == POST_AND_STOP_CALLS);
}

#[test]
fn a_watched_commit_makes_at_most_ten_network_calls_and_one_per_poll() {
    let repo = Repo::new(SLOW);
    let state = repo.clone.path().join(STATE_ROOT);
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(state.join(EXPECTED), PLANTED).unwrap();
    repo.hook(POST, COMMIT);
    let settled = repo.settled();
    let calls = std::fs::read_to_string(repo.clone.path().join(CALLS)).unwrap_or_default();
    let polls: BTreeSet<&str> = calls.lines().collect();
    let learned = std::fs::read_to_string(state.join(EXPECTED)).unwrap_or_default();
    assert!(settled.contains(TIMED_OUT) && (1..=CALL_LOG_MAX_LEN).contains(&calls.lines().count()));
    assert_eq!(polls.len(), calls.lines().count());
    assert_eq!([calls.lines().next(), Some(learned.as_str())], [Some(PLANTED), Some(LEARNED)]);
}
