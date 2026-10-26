use super::{
    BLOCK, CI_WAIT, COMMIT, FAILED_EVENT, FAILURE, Git, JOB_FAILED, NEXT_COMMIT, POST, Planted, Repo, STOP, SUCCESS,
};

const STATUS: &str = "git status";
const ELSEWHERE: [&str; 4] = ["push", "-q", "origin", "HEAD:elsewhere"];
const DETACH: [&str; 3] = ["checkout", "-q", "--detach"];
const BACK: [&str; 3] = ["checkout", "-q", "feature"];
const NOT_PUSHED: &str = "is not pushed";
const UNWRITABLE: &str = "CI state unwritable";
const SESSION_DIR: &str = ".git/meta-ci/ABCDEF1234";
const PASSED: &str = "passed";

#[test]
fn stop_judges_every_pushed_commit_and_a_dead_watcher_restarts() {
    let repo = Repo::new(FAILURE);
    Planted::running(&repo);
    Git::run(repo.clone.path(), &ELSEWHERE);
    Git::run(repo.clone.path(), &DETACH);
    let detached = repo.hook(STOP, STATUS);
    let waited = repo.session_command(CI_WAIT).0;
    Planted::refusal(repo.origin.path());
    Git::run(repo.clone.path(), &BACK);
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    repo.hook(POST, COMMIT);
    let rejected = repo.hook(STOP, COMMIT);
    let open = Planted::locked(&repo.clone.path().join(SESSION_DIR));
    let unwritable = repo.hook(STOP, COMMIT);
    std::fs::set_permissions(repo.clone.path().join(SESSION_DIR), open).unwrap();
    assert!(
        detached.contains(BLOCK)
            && waited.contains(JOB_FAILED)
            && rejected.contains(NOT_PUSHED)
            && unwritable.contains(UNWRITABLE)
            && repo.settled().contains(FAILED_EVENT)
    );
}

#[test]
fn one_commit_gets_one_watcher() {
    let repo = Repo::new(SUCCESS);
    let twins = [Planted::twin(&repo), Planted::twin(&repo)].map(|child| child.wait_with_output().unwrap());
    let passes = twins.iter().filter(|twin| String::from_utf8_lossy(&twin.stdout).contains(PASSED)).count();
    assert_eq!(passes, 1);
}
