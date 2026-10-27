use std::collections::BTreeSet;

use super::seeded_log::{SeededLog, TASK};
use super::{BLOCK, COMMIT, Git, NEXT_COMMIT, POST, Planted, Repo, STOP, SUCCESS, TIMED_OUT_STOP};

const TIMED_OUT: &str = "timed-out";
const SLOW: &str = "slow";
const HOUR: &str = "timeout_millis = 3600000\n";
const CALLS: &str = "calls";
const CALLS_PER_COMMIT: usize = 10;
const DETACH: [&str; 3] = ["checkout", "-q", "--detach"];
const PUSH_HEAD: [&str; 4] = ["push", "-q", "origin", "HEAD:refs/heads/pushed"];
const WAIT_ON_CI: [&str; 4] = ["wait", TASK, "--on", "ci"];
const TASKLESS: &str = "then run `meta wait <task> --on ci` here";

struct Unresolved;

impl Unresolved {
    fn pushed(repo: &Repo) -> String {
        Git::run(repo.clone.path(), &PUSH_HEAD);
        Planted::settled(repo, TIMED_OUT);
        repo.hook(STOP, COMMIT)
    }
}

#[test]
fn on_a_detached_head_a_wait_covers_only_the_head_it_was_recorded_at() {
    let repo = Repo::new(SUCCESS);
    SeededLog::CLAIMED.plant(repo.origin.path());
    Git::run(repo.clone.path(), &DETACH);
    let unrecorded = Unresolved::pushed(&repo);
    repo.meta(&WAIT_ON_CI);
    let recorded = repo.hook(STOP, COMMIT);
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    let moved_on = Unresolved::pushed(&repo);
    assert!(unrecorded.contains(TIMED_OUT_STOP) && unrecorded.contains(TASKLESS));
    assert!(serde_json::from_str::<serde_json::Value>(&unrecorded).is_ok() && !recorded.contains(BLOCK));
    assert!(moved_on.contains(TASKLESS));
}

#[test]
fn a_watched_commit_makes_at_most_ten_network_calls_and_one_per_poll() {
    let repo = Repo::limited(SLOW, HOUR);
    repo.hook(POST, COMMIT);
    let settled = repo.settled();
    let calls = std::fs::read_to_string(repo.clone.path().join(CALLS)).unwrap_or_default();
    let polls: BTreeSet<&str> = calls.lines().collect();
    assert!(settled.contains(TIMED_OUT) && (1..=CALLS_PER_COMMIT).contains(&calls.lines().count()));
    assert_eq!(polls.len(), calls.lines().count());
}
