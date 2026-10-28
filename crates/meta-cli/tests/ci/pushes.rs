use super::scenes::Scene;
use super::{
    BLOCK, CI_WAIT, COMMIT, FAILED_EVENT, FAILURE, Git, JOB_FAILED, NEXT_COMMIT, POST, PRE, Planted, Remote, Repo,
    STOP, SUCCESS, Session,
};

const STATUS: &str = "git status";
const ELSEWHERE: &str = "elsewhere";
const DETACH: [&str; 3] = ["checkout", "-q", "--detach"];
const BACK: [&str; 3] = ["checkout", "-q", "feature"];
const NOT_PUSHED: &str = "is not pushed";
const UNWRITABLE: &str = "CI state unwritable";
const SESSION_DIR: &str = ".git/meta-ci/ABCDEF1234";
const PASSED: &str = "passed";
const STUCK: &str = "STUCK9";
const START_DIR: &str = "meta-sessions/STUCK9";
const DENIED: &str = "\"permissionDecision\":\"deny\"";

pub struct StopJudgesPushedCommits;

impl Scene for StopJudgesPushedCommits {
    fn hold() {
        let repo = Repo::new(FAILURE);
        Planted::settled(&repo, Planted::RUNNING);
        Remote::pushed(&repo, ELSEWHERE);
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
}

pub struct OneCommitOneWatcher;

impl Scene for OneCommitOneWatcher {
    fn hold() {
        let repo = Repo::new(SUCCESS);
        let twins = [Planted::twin(&repo), Planted::twin(&repo)].map(|child| child.wait_with_output().unwrap());
        let passes = twins.iter().filter(|twin| String::from_utf8_lossy(&twin.stdout).contains(PASSED)).count();
        assert_eq!(passes, 1);
    }
}

pub struct UnwritableSessionStart;

impl Scene for UnwritableSessionStart {
    fn hold() {
        let repo = Repo::new(SUCCESS);
        let start = repo.sessions.path().join(START_DIR);
        std::fs::create_dir_all(&start).unwrap();
        let open = Planted::locked(&start);
        let refused = Session { repo: &repo, cwd: repo.clone.path(), id: STUCK }.hook(PRE, COMMIT);
        std::fs::set_permissions(&start, open).unwrap();
        assert!(refused.contains(DENIED) && refused.contains(START_DIR));
    }
}
