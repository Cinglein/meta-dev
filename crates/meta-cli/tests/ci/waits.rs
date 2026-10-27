use super::{BLOCK, COMMIT, Git, PUSH_WORDS, Planted, Repo, STOP, SUCCESS, TASKLESS_WAY_OUT, TIMED_OUT_STOP};

const OPEN: [&str; 4] = ["open", "stalled CI", "--division", "features-and-maintenance"];
const TIMED_OUT: &str = "timed-out";

struct TaskBranch(String);

impl TaskBranch {
    const PREFIX: &'static str = "task-";

    fn claimed(repo: &Repo) -> Self {
        let opened = repo.meta(&OPEN).0;
        let task = opened.split_whitespace().find(|word| word.starts_with(Self::PREFIX)).unwrap().to_owned();
        Git::run(repo.clone.path(), &["checkout", "-q", "-b", &task]);
        repo.meta(&["claim", &task]);
        Self(task)
    }

    fn wait_on_ci(&self, repo: &Repo) {
        repo.meta(&["wait", &self.0, "--on", "ci"]);
    }

    fn way_out(&self) -> String {
        format!("or run `meta wait {} --on ci` to stop with the wait on record", self.0)
    }
}

#[test]
fn only_a_recorded_wait_on_ci_lets_an_unresolved_run_stop() {
    let repo = Repo::new(SUCCESS);
    Git::run(repo.clone.path(), &PUSH_WORDS);
    Planted::settled(&repo, TIMED_OUT);
    let taskless = repo.hook(STOP, COMMIT);
    let branch = TaskBranch::claimed(&repo);
    let unrecorded = repo.hook(STOP, COMMIT);
    branch.wait_on_ci(&repo);
    let recorded = repo.hook(STOP, COMMIT);
    assert!(taskless.contains(TIMED_OUT_STOP) && taskless.contains(TASKLESS_WAY_OUT));
    assert!(unrecorded.contains(&branch.way_out()) && serde_json::from_str::<serde_json::Value>(&unrecorded).is_ok());
    assert!(!recorded.contains(BLOCK));
}
