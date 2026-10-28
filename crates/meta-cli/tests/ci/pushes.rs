use super::{COMMIT, PRE, Planted, Repo, SUCCESS, Session};

const PASSED: &str = "passed";
const STUCK: &str = "STUCK9";
const START_DIR: &str = "meta-sessions/STUCK9";
const DENIED: &str = "\"permissionDecision\":\"deny\"";

#[test]
fn one_commit_gets_one_watcher() {
    let repo = Repo::new(SUCCESS);
    let twins = [Planted::twin(&repo), Planted::twin(&repo)].map(|child| child.wait_with_output().unwrap());
    let passes = twins.iter().filter(|twin| String::from_utf8_lossy(&twin.stdout).contains(PASSED)).count();
    assert_eq!(passes, 1);
}

#[test]
fn an_unwritable_session_start_refuses_the_writing_command() {
    let repo = Repo::new(SUCCESS);
    let start = repo.sessions.path().join(START_DIR);
    std::fs::create_dir_all(&start).unwrap();
    let open = Planted::locked(&start);
    let refused = Session { repo: &repo, cwd: repo.clone.path(), id: STUCK }.hook(PRE, COMMIT);
    std::fs::set_permissions(&start, open).unwrap();
    assert!(refused.contains(DENIED) && refused.contains(START_DIR));
}
