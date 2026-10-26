use super::{BLOCK, COMMIT, FAILURE, Git, NEXT_COMMIT, PASSED_EVENT, POST, Planted, Repo, STOP, SUCCESS};

const STACK_PUSH: &str = "gh stack push";
const ELSEWHERE: [&str; 4] = ["push", "-q", "origin", "HEAD:elsewhere"];
const DETACH: [&str; 3] = ["checkout", "-q", "--detach"];
const BACK: [&str; 3] = ["checkout", "-q", "feature"];
const NOT_PUSHED: &str = "is not pushed";
const UNWRITABLE: &str = "CI state unwritable";
const SESSION_DIR: &str = ".git/meta-ci/ABCDEF1234";
const PASSED: &str = "passed";

#[test]
fn pushes_to_other_names_rejected_pushes_and_unwritable_state_block_stop() {
    let repo = Repo::new(FAILURE);
    Git::run(repo.clone.path(), &ELSEWHERE);
    repo.hook(POST, STACK_PUSH);
    Git::run(repo.clone.path(), &DETACH);
    let detached = repo.hook(STOP, COMMIT);
    Planted::refusal(repo.origin.path());
    Git::run(repo.clone.path(), &BACK);
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    repo.hook(POST, COMMIT);
    let rejected = repo.hook(STOP, COMMIT);
    let open = Planted::locked(&repo.clone.path().join(SESSION_DIR));
    let unwritable = repo.hook(STOP, COMMIT);
    std::fs::set_permissions(repo.clone.path().join(SESSION_DIR), open).unwrap();
    repo.settled();
    assert!(detached.contains(BLOCK) && rejected.contains(NOT_PUSHED) && unwritable.contains(UNWRITABLE));
}

#[test]
fn a_dead_watcher_restarts_and_one_commit_gets_one_watcher() {
    let [dead, twinned] = [SUCCESS, SUCCESS].map(Repo::new);
    Planted::running(&dead);
    dead.hook(POST, COMMIT);
    let twins = [Planted::twin(&twinned), Planted::twin(&twinned)].map(|child| child.wait_with_output().unwrap());
    let passes = twins.iter().filter(|twin| String::from_utf8_lossy(&twin.stdout).contains(PASSED)).count();
    assert!(dead.settled().contains(PASSED_EVENT) && passes == 1);
}
