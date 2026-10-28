use super::{BLOCK, CI_WAIT, COMMIT, DEADLINE, EXPECTED, Git, NEXT_COMMIT, PASSED_EVENT, POST, Repo, STOP, SUCCESS};
const NAMELESS: &str = "origin.git";
const FIRST_POLL: &str = "30";
const UNCHECKED_EVENT: &str = ": unchecked after";
const UNCHECKED_STOP: &str = "couldn't check CI for your last push";
const TASKLESS: &str = "then run `meta wait <task> --on ci` here";
const LEARNED: &str = "422000";

#[test]
fn an_unreadable_remote_settles_the_watch_unchecked_and_the_last_poll_still_ends_before_the_real_deadline() {
    let repo = Repo::limited(SUCCESS, DEADLINE);
    let origin = repo.origin.to_str().unwrap().to_owned();
    std::os::unix::fs::symlink(&repo.origin, repo.clone.path().join(NAMELESS)).unwrap();
    Git::run(repo.clone.path(), &["remote", "set-url", "origin", NAMELESS]);
    repo.hook(POST, COMMIT);
    let unchecked = repo.settled();
    let stopped = repo.hook(STOP, COMMIT);
    let unchecked_wait = repo.session_command(CI_WAIT).1;
    Git::run(repo.clone.path(), &["remote", "set-url", "origin", origin.as_str()]);
    std::fs::write(repo.clone.path().join(EXPECTED), FIRST_POLL).unwrap();
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    repo.answered();
    repo.hook(POST, COMMIT);
    let retried = repo.settled_past(&unchecked);
    let shown = [unchecked.as_str(), stopped.as_str()].concat();
    assert!(
        [UNCHECKED_EVENT, BLOCK, UNCHECKED_STOP, TASKLESS].iter().all(|told| shown.contains(told))
            && !unchecked_wait.success()
    );
    let learned = std::fs::read_to_string(repo.clone.path().join(EXPECTED)).unwrap();
    assert!(retried.matches(UNCHECKED_EVENT).count() == 1 && learned == LEARNED, "{learned}");
    assert!(retried.strip_prefix(unchecked.as_str()).is_some_and(|new| new.contains(PASSED_EVENT)), "{retried}");
}
