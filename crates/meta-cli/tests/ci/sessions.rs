use std::path::Path;

use super::{
    BLOCK, COMMIT, FAILURE, FailingHead, Git, NEXT_COMMIT, POST, PRE, PUSH_WORDS, Planted, Repo, STOP, Session,
    StartFile,
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
const FUTURE: &str = "@4000000000 +0000";
const FUTURE_START: &str = "4000000000";

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

#[test]
fn other_sessions_commits_and_a_read_only_state_dir_never_block() {
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
        theirs.hook(STOP, COMMIT).contains(BLOCK) && !passed.iter().chain([&locked]).any(|stop| stop.contains(BLOCK))
    );
}
