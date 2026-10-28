use super::seeded_log::{SeededLog, TASK};
use super::{BLOCK, COMMIT, Git, NEXT_COMMIT, Planted, Repo, SHELL, STOP, SUCCESS, TIMED_OUT_STOP};

const TIMED_OUT: &str = "timed-out";
const COUNTING_GIT: &str =
    "#!/bin/sh\necho \"$3\" >> \"$FAKE_GH_DIR/git-calls\"\nPATH=\"${PATH#*:}\" exec git \"$@\"\n";
const WRAPPER: &str = "bin/git";
const CALLS: &str = "git-calls";
const STOP_CALLS: [&str; 5] = ["log", "ls-remote", "merge-base", "rev-list", "rev-parse"];
const DETACH: [&str; 3] = ["checkout", "-q", "--detach"];
const PUSH_HEAD: [&str; 4] = ["push", "-q", "origin", "HEAD:refs/heads/pushed"];
const WAIT_ON_CI: [&str; 4] = ["wait", TASK, "--on", "ci"];
const TASKLESS: &str = "then run `meta wait <task> --on ci` here";

struct Unresolved;

impl Unresolved {
    fn pushed(repo: &Repo) -> String {
        Git::run(repo.clone.path(), &PUSH_HEAD);
        Planted::settled(repo, TIMED_OUT);
        repo.hook(STOP, COMMIT)
    }
}

#[test]
fn on_a_detached_head_a_wait_covers_only_the_head_it_was_recorded_at() {
    let repo = Repo::new(SUCCESS);
    SeededLog::CLAIMED.plant(repo.origin.path());
    Git::run(repo.clone.path(), &DETACH);
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
fn a_typical_stop_lists_the_remote_once_in_five_git_calls() {
    let repo = Repo::new(SUCCESS);
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    let wrapper = repo.clone.path().join(WRAPPER);
    std::fs::write(&wrapper, COUNTING_GIT).unwrap();
    std::fs::set_permissions(&wrapper, std::fs::metadata(SHELL).unwrap().permissions()).unwrap();
    let stop = Unresolved::pushed(&repo);
    let calls = std::fs::read_to_string(repo.clone.path().join(CALLS)).unwrap();
    let mut calls: Vec<&str> = calls.lines().collect();
    calls.sort_unstable();
    assert!(stop.contains(TIMED_OUT_STOP) && stop.contains(TASKLESS));
    assert_eq!(calls, STOP_CALLS);
}
