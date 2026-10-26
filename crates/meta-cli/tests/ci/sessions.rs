use std::path::Path;

use super::{
    BLOCK, COMMIT, FAILURE, FailingHead, Git, NEXT_COMMIT, POST, PRE, PUSH_WORDS, Planted, QUICK, Repo, STOP, Scenario,
    Session, StartFile,
};

const MINE: &str = "MINE1";
const THEIRS: &str = "THEIRS2";
const READER: &str = "READER3";
const PULLER: &str = "PULLER4";
const BUILD: &str = "cargo xtask ci";
const PULL: &str = "git pull";
const CONFIG_FILE: &str = "meta.toml";
const WORKTREE: &str = "other";
const ADD_WORKTREE: [&str; 6] = ["worktree", "add", "-q", WORKTREE, "-b", WORKTREE];
const FEATURE: &str = "feature";
const PULL_WORDS: [&str; 4] = ["pull", "-q", "origin", WORKTREE];
const STATE_ROOT: &str = ".git/meta-ci";
const FUTURE: &str = "@4000000001 +0000";
const FUTURE_START: &str = "4000000000";
const SAME: &str = "SAME5";
const NOW: &str = "NOW6";
const SAME_AT: &str = "4000000500";
const SLOW_REMOTE: [&str; 3] = ["config", "remote.origin.uploadpack", "sleep 5; git-upload-pack"];
const SLOW: std::time::Duration = std::time::Duration::from_secs(3);
const UNREACHABLE: &str = "remote unreachable";

struct Reviewer;

impl Reviewer {
    fn clone_of(repo: &Repo) -> tempfile::TempDir {
        let reviewer = tempfile::tempdir().unwrap();
        let origin = repo.origin.path().to_str().unwrap();
        Git::run(reviewer.path(), &["clone", "-q", "-b", FEATURE, origin, "."]);
        std::fs::copy(repo.clone.path().join(CONFIG_FILE), reviewer.path().join(CONFIG_FILE)).unwrap();
        reviewer
    }

    fn plant_future_start(repo: &Repo) {
        StartFile::plant(repo.sessions.path(), THEIRS, FUTURE_START);
    }

    fn worktree(repo: &Repo) -> std::path::PathBuf {
        Git::run(repo.clone.path(), &ADD_WORKTREE);
        let other = repo.clone.path().join(WORKTREE);
        std::fs::copy(repo.clone.path().join(CONFIG_FILE), other.join(CONFIG_FILE)).unwrap();
        other
    }
}

#[derive(Debug)]
pub struct SlowRemote;

impl Scenario for SlowRemote {
    fn run() {
        let repo = Repo::limited(FAILURE, QUICK);
        let clone: &Path = repo.clone.path();
        StartFile::plant(repo.sessions.path(), SAME, SAME_AT);
        Git::run_at(clone, &NEXT_COMMIT, SAME_AT);
        Git::run(clone, &PUSH_WORDS);
        Git::run(clone, &SLOW_REMOTE);
        let started = std::time::Instant::now();
        let slow = repo.hook(STOP, COMMIT);
        let took = started.elapsed();
        let same = Session { repo: &repo, cwd: clone, id: SAME }.hook(STOP, COMMIT);
        assert!(slow.contains(UNREACHABLE) && took < SLOW && !same.contains(BLOCK));
    }
}

#[derive(Debug)]
pub struct StartSecond;

impl Scenario for StartSecond {
    fn run() {
        let repo = Repo::new(FAILURE);
        let clone: &Path = repo.clone.path();
        let now = Session { repo: &repo, cwd: clone, id: NOW };
        now.hook(PRE, COMMIT);
        let second = StartFile::second_of(repo.sessions.path(), NOW);
        Git::run_at(clone, &NEXT_COMMIT, &second);
        Git::run(clone, &PUSH_WORDS);
        assert!(now.hook(STOP, COMMIT).contains(BLOCK));
    }
}

#[derive(Debug)]
pub struct OtherSessions;

impl Scenario for OtherSessions {
    fn run() {
        let repo = Repo::new(FAILURE);
        let other = Reviewer::worktree(&repo);
        Git::run(repo.clone.path(), &PUSH_WORDS);
        let reviewer = Reviewer::clone_of(&repo);
        Reviewer::plant_future_start(&repo);
        let clone: &Path = repo.clone.path();
        let cwds = [(clone, MINE), (&other, THEIRS), (clone, READER), (reviewer.path(), PULLER)];
        let [mine, theirs, reader, puller] = cwds.map(|(cwd, id)| Session { repo: &repo, cwd, id });
        let _: Vec<String> = [&mine, &theirs, &puller].iter().map(|session| session.hook(PRE, COMMIT)).collect();
        Git::run_at(&other, &NEXT_COMMIT, FUTURE);
        FailingHead::copy_from(&repo, &other);
        let _: Vec<String> = [&theirs, &mine].iter().map(|session| session.hook(POST, COMMIT)).collect();
        Git::run_at(reviewer.path(), &PULL_WORDS, FUTURE);
        puller.hook(POST, PULL);
        let passed = [mine.hook(STOP, BUILD), puller.hook(STOP, PULL)];
        let open = Planted::locked(&clone.join(STATE_ROOT));
        let locked = reader.hook(STOP, BUILD);
        std::fs::set_permissions(clone.join(STATE_ROOT), open).unwrap();
        assert!(
            theirs.hook(STOP, COMMIT).contains(BLOCK)
                && !passed.iter().chain([&locked]).any(|stop| stop.contains(BLOCK))
        );
    }
}
