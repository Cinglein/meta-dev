use meta_door::{Change, Door};

use super::seeded_log::{SeededLog, TASK};
use super::{
    BLOCK, COMMIT, Calls, DEADLINE, FailingHead, Git, LAST, NEXT_COMMIT, PASSED_EVENT, POST, Planted, Repo, SHELL,
    STOP, SUCCESS, TIMED_OUT_STOP,
};

const TIMED_OUT: &str = "timed-out";
const NAMELESS: &str = "origin.git";
const FIRST_POLL: &str = "80";
const GIT_FAILED: &str = "simulated git failure";
const UNCHECKED_EVENT: &str = ": unchecked after";
const UNCHECKED_STOP: &str = "couldn't check CI for your last push";
const DETACH: [&str; 3] = ["checkout", "-q", "--detach"];
const PUSH_HEAD: [&str; 4] = ["push", "-q", "origin", "HEAD:refs/heads/pushed"];
const WAIT_ON_CI: [&str; 4] = ["wait", TASK, "--on", "ci"];
const TASKLESS: &str = "then run `meta wait <task> --on ci` here";
const POST_AND_STOP_CALLS: [&str; 9] =
    ["cat-file", "log", "log", "ls-remote", "merge-base", "rev-list", "rev-list", "rev-parse", "rev-parse"];

struct Unresolved;

impl Unresolved {
    fn pushed(repo: &Repo) -> String {
        Git::run(repo.clone.path(), &PUSH_HEAD);
        Planted::settled(repo, TIMED_OUT);
        repo.hook(STOP, COMMIT)
    }
}

struct Counted;

impl Counted {
    const GIT: &'static str =
        "#!/bin/sh\necho \"$3\" >> \"$FAKE_GH_DIR/git-calls\"\nPATH=\"${PATH#*:}\" exec git \"$@\"\n";
    const FLAKY: &'static str = "#!/bin/sh\n[ \"$3\" = remote ] && [ ! -e \"$FAKE_GH_DIR/flaked\" ] && : > \"$FAKE_GH_DIR/flaked\" && echo \"fatal: simulated git failure\" >&2 && exit 1\nPATH=\"${PATH#*:}\" exec git \"$@\"\n";
    const WRAPPER: &'static str = "bin/git";
    const CALLS: &'static str = "git-calls";

    fn install(repo: &Repo) {
        Self::plant(repo, Self::GIT);
    }

    fn plant(repo: &Repo, script: &str) {
        let wrapper = repo.clone.path().join(Self::WRAPPER);
        Door::write(&wrapper, script).unwrap();
        let executable = Door::described(Door::canonical(SHELL).unwrap()).unwrap().permissions();
        Door::changed(&wrapper, Change::Permissions(executable)).unwrap();
    }

    fn calls(repo: &Repo) -> Vec<String> {
        let text = Door::read(repo.clone.path().join(Self::CALLS)).unwrap_or_default();
        let mut calls: Vec<String> = text.lines().map(str::to_owned).collect();
        calls.sort_unstable();
        calls
    }
}

#[test]
fn on_a_detached_head_a_wait_covers_only_its_head_and_a_typical_stop_lists_the_remote_once() {
    let repo = Repo::new(SUCCESS);
    SeededLog::CLAIMED.plant(repo.origin.path());
    Git::run(repo.clone.path(), &DETACH);
    let unrecorded = Unresolved::pushed(&repo);
    repo.meta(&WAIT_ON_CI);
    let recorded = repo.hook(STOP, COMMIT);
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    Counted::install(&repo);
    repo.hook(POST, COMMIT);
    let moved_on = Unresolved::pushed(&repo);
    assert!(unrecorded.contains(TIMED_OUT_STOP) && unrecorded.contains(TASKLESS));
    assert!(serde_json::from_str::<serde_json::Value>(&unrecorded).is_ok() && !recorded.contains(BLOCK));
    let calls = Counted::calls(&repo);
    assert!(moved_on.contains(TASKLESS) && calls == POST_AND_STOP_CALLS, "{calls:?}");
}

#[test]
fn an_unreadable_remote_settles_the_watch_unchecked_and_the_last_poll_still_ends_before_the_real_deadline() {
    let repo = Repo::limited(LAST, DEADLINE);
    let origin = repo.origin.path().to_str().unwrap().to_owned();
    Git::run(repo.clone.path(), &["clone", "-q", "--bare", origin.as_str(), NAMELESS]);
    Git::run(repo.clone.path(), &["remote", "set-url", "origin", NAMELESS]);
    repo.hook(POST, COMMIT);
    let unchecked = repo.settled();
    let stopped = repo.hook(STOP, COMMIT);
    Git::run(repo.clone.path(), &["remote", "set-url", "origin", origin.as_str()]);
    Counted::plant(&repo, Counted::FLAKY);
    Calls::expect(&repo, FIRST_POLL);
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    FailingHead::copy_from(&repo, repo.clone.path());
    repo.hook(POST, COMMIT);
    let retried = repo.settled_past(&unchecked);
    assert!(unchecked.contains(UNCHECKED_EVENT) && stopped.contains(BLOCK) && stopped.contains(UNCHECKED_STOP));
    assert!(retried.contains(GIT_FAILED) && retried.matches(UNCHECKED_EVENT).count() == 1);
    assert!(retried.strip_prefix(unchecked.as_str()).is_some_and(|new| new.contains(PASSED_EVENT)), "{retried}");
}
