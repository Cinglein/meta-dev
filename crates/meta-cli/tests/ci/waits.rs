use super::seeded_log::{SeededLog, TASK};
use super::{BLOCK, COMMIT, Git, NEXT_COMMIT, PUSH_WORDS, Planted, Repo, STOP, SUCCESS, TIMED_OUT_STOP};

const TIMED_OUT: &str = "timed-out";
const TASK_BRANCH: [&str; 4] = ["checkout", "-q", "-b", TASK];
const DETACH: [&str; 3] = ["checkout", "-q", "--detach"];
const PUSH_HEAD: [&str; 4] = ["push", "-q", "origin", "HEAD:refs/heads/pushed"];
const WAIT_ON_CI: [&str; 4] = ["wait", TASK, "--on", "ci"];
const DONE: [&str; 2] = ["done", TASK];
const TASKLESS: &str = "then run `meta wait <task> --on ci` here";
const RESUME_FIRST: &str = "run `meta resume task-00000000000000a1` first";
const REOPEN: &str = "run `meta reopen task-00000000000000a1`";

struct Unresolved;

impl Unresolved {
    fn pushed(repo: &Repo) -> String {
        Git::run(repo.clone.path(), &PUSH_HEAD);
        Planted::settled(repo, TIMED_OUT);
        repo.hook(STOP, COMMIT)
    }
}

#[test]
fn on_a_taskless_branch_a_wait_covers_only_the_head_it_was_recorded_at() {
    let repo = Repo::new(SUCCESS);
    SeededLog::CLAIMED.plant(repo.origin.path());
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
fn on_a_detached_head_a_recorded_wait_lets_stop_pass() {
    let repo = Repo::new(SUCCESS);
    SeededLog::CLAIMED.plant(repo.origin.path());
    Git::run(repo.clone.path(), &DETACH);
    let unrecorded = Unresolved::pushed(&repo);
    repo.meta(&WAIT_ON_CI);
    assert!(unrecorded.contains(TASKLESS) && !repo.hook(STOP, COMMIT).contains(BLOCK));
}

#[test]
fn a_branch_task_waiting_on_an_older_head_must_resume_and_a_done_one_reopen() {
    let repo = Repo::new(SUCCESS);
    SeededLog::CLAIMED.plant(repo.origin.path());
    Git::run(repo.clone.path(), &TASK_BRANCH);
    Git::run(repo.clone.path(), &PUSH_WORDS);
    repo.meta(&WAIT_ON_CI);
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    let waiting = Unresolved::pushed(&repo);
    repo.meta(&DONE);
    let done = repo.hook(STOP, COMMIT);
    assert!(waiting.contains(RESUME_FIRST) && done.contains(REOPEN));
}
