use std::path::Path;

use super::{BLOCK, CI_WAIT, COMMIT, EXPECTED, NEXT_COMMIT, PASSED_EVENT, POST, Planted, Repo, STOP, SUCCESS};
const NAMELESS: &str = "origin.git";
const UNHURRIED: &str = "expected_millis = 0\n";
const UNCHECKED_EVENT: &str = ": unchecked after";
const UNCHECKED_STOP: &str = "couldn't check CI for your last push";
const TASKLESS: &str = "then run `meta wait <task> --on ci` here";
const LEARNED: &str = "422000";

#[test]
fn an_unreadable_remote_settles_the_hook_spawned_watch_unchecked_and_a_fixed_remote_passes_and_learns_the_run() {
    let repo = Repo::limited(SUCCESS, UNHURRIED);
    std::os::unix::fs::symlink(&repo.origin, repo.clone.path().join(NAMELESS)).unwrap();
    repo.git().pointed(Path::new(NAMELESS));
    repo.hook(POST, COMMIT);
    let unchecked = repo.settled();
    let stopped = repo.hook(STOP, COMMIT);
    let waited = repo.meta(&[CI_WAIT]).1;
    repo.git().pointed(&repo.origin);
    repo.git().committed(NEXT_COMMIT);
    repo.answered();
    Planted::twin(&repo).wait().unwrap();
    let retried = repo.settled_past(&unchecked);
    let shown = [unchecked.as_str(), stopped.as_str()].concat();
    assert!([UNCHECKED_EVENT, BLOCK, UNCHECKED_STOP, TASKLESS].iter().all(|told| shown.contains(told)));
    let learned = std::fs::read_to_string(repo.clone.path().join(EXPECTED)).unwrap();
    assert!(retried.matches(UNCHECKED_EVENT).count() == 1 && learned == LEARNED && !waited.success(), "{learned}");
    assert!(retried.strip_prefix(unchecked.as_str()).is_some_and(|new| new.contains(PASSED_EVENT)), "{retried}");
}
