use super::{BLOCK, COMMIT, DEADLINE, Git, NEXT_COMMIT, PASSED_EVENT, POST, Repo, STOP, SUCCESS};

const EXPECTED: &str = ".git/meta-ci/expected";
const NAMELESS: &str = "origin.git";
const FIRST_POLL: &str = "30";
const UNCHECKED_EVENT: &str = ": unchecked after";
const UNCHECKED_STOP: &str = "couldn't check CI for your last push";

#[test]
fn an_unreadable_remote_settles_the_watch_unchecked_and_the_last_poll_still_ends_before_the_real_deadline() {
    let repo = Repo::limited(SUCCESS, DEADLINE);
    let origin = repo.origin.to_str().unwrap().to_owned();
    std::os::unix::fs::symlink(&repo.origin, repo.clone.path().join(NAMELESS)).unwrap();
    Git::run(repo.clone.path(), &["remote", "set-url", "origin", NAMELESS]);
    repo.hook(POST, COMMIT);
    let unchecked = repo.settled();
    let stopped = repo.hook(STOP, COMMIT);
    Git::run(repo.clone.path(), &["remote", "set-url", "origin", origin.as_str()]);
    std::fs::write(repo.clone.path().join(EXPECTED), FIRST_POLL).unwrap();
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    repo.answered();
    repo.hook(POST, COMMIT);
    let retried = repo.settled_past(&unchecked);
    assert!(unchecked.contains(UNCHECKED_EVENT) && stopped.contains(BLOCK) && stopped.contains(UNCHECKED_STOP));
    assert_eq!(retried.matches(UNCHECKED_EVENT).count(), 1);
    assert!(retried.strip_prefix(unchecked.as_str()).is_some_and(|new| new.contains(PASSED_EVENT)), "{retried}");
}
