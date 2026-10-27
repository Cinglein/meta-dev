use super::seeded_log::{SeededLog, TASK};
use super::{BLOCK, COMMIT, Git, PUSH_WORDS, Planted, Repo, STOP, SUCCESS, TIMED_OUT_STOP};

const TIMED_OUT: &str = "timed-out";
const TASK_BRANCH: [&str; 4] = ["checkout", "-q", "-b", TASK];
const WAIT_ON_CI: [&str; 4] = ["wait", TASK, "--on", "ci"];
const WAY_OUT: &str = "; or run `meta wait task-00000000000000a1 --on ci` to stop with the wait on record";

#[test]
fn only_a_recorded_wait_on_ci_lets_an_unresolved_run_stop() {
    let repo = Repo::new(SUCCESS);
    Git::run(repo.clone.path(), &PUSH_WORDS);
    Planted::settled(&repo, TIMED_OUT);
    SeededLog::CLAIMED.plant(repo.origin.path());
    Git::run(repo.clone.path(), &TASK_BRANCH);
    let unrecorded = repo.hook(STOP, COMMIT);
    repo.meta(&WAIT_ON_CI);
    let recorded = repo.hook(STOP, COMMIT);
    assert!(unrecorded.contains(TIMED_OUT_STOP) && unrecorded.contains(WAY_OUT));
    assert!(serde_json::from_str::<serde_json::Value>(&unrecorded).is_ok() && !recorded.contains(BLOCK));
}
