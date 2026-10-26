use std::path::Path;

use super::{
    BLOCK, CI_WAIT, COMMIT, FAILURE, FailingHead, Git, NEXT_COMMIT, POST, PRE, PUSH_WORDS, Planted, Repo, STOP, Session,
};

const MINE: &str = "MINE1";
const THEIRS: &str = "THEIRS2";
const READER: &str = "READER3";
const BUILD: &str = "cargo xtask ci";
const PULL: &str = "git pull";
const CONFIG_FILE: &str = "meta.toml";
const WORKTREE: &str = "other";
const ADD_WORKTREE: [&str; 6] = ["worktree", "add", "-q", WORKTREE, "-b", WORKTREE];
const FEATURE: &str = "feature";
const PULL_WORDS: [&str; 2] = ["pull", "-q"];
const STATE_ROOT: &str = ".git/meta-ci";

struct Reviewer;

impl Reviewer {
    fn clone_of(repo: &Repo) -> tempfile::TempDir {
        let reviewer = tempfile::tempdir().unwrap();
        let origin = repo.origin.path().to_str().unwrap();
        Git::run(reviewer.path(), &["clone", "-q", "-b", FEATURE, origin, "."]);
        std::fs::copy(repo.clone.path().join(CONFIG_FILE), reviewer.path().join(CONFIG_FILE)).unwrap();
        reviewer
    }

    fn worktree(repo: &Repo) -> std::path::PathBuf {
        Git::run(repo.clone.path(), &ADD_WORKTREE);
        let other = repo.clone.path().join(WORKTREE);
        std::fs::copy(repo.clone.path().join(CONFIG_FILE), other.join(CONFIG_FILE)).unwrap();
        other
    }
}

#[test]
fn a_push_no_hook_saw_is_still_judged() {
    let repo = Repo::new(FAILURE);
    Git::run(repo.clone.path(), &PUSH_WORDS);
    let stop = repo.hook(STOP, BUILD);
    let waited = repo.session_command(CI_WAIT);
    repo.settled();
    assert!(stop.contains(BLOCK) && !waited.1.success());
}

#[test]
fn another_worktrees_commit_and_a_read_only_state_dir_never_block() {
    let repo = Repo::new(FAILURE);
    let other = Reviewer::worktree(&repo);
    let clone: &Path = repo.clone.path();
    let [mine, theirs, reader] =
        [(clone, MINE), (&other, THEIRS), (clone, READER)].map(|(cwd, id)| Session { repo: &repo, cwd, id });
    mine.hook(PRE, BUILD);
    theirs.hook(PRE, COMMIT);
    Git::run(&other, &NEXT_COMMIT);
    FailingHead::set(&repo, &other);
    theirs.hook(POST, COMMIT);
    mine.hook(POST, BUILD);
    let [mine, theirs] = [mine.hook(STOP, BUILD), theirs.hook(STOP, COMMIT)];
    let open = Planted::locked(&clone.join(STATE_ROOT));
    let locked = reader.hook(STOP, BUILD);
    std::fs::set_permissions(clone.join(STATE_ROOT), open).unwrap();
    assert!(!mine.contains(BLOCK) && theirs.contains(BLOCK) && !locked.contains(BLOCK));
}

#[test]
fn a_reviewers_pull_of_a_red_branch_never_blocks() {
    let repo = Repo::new(FAILURE);
    Git::run(repo.clone.path(), &PUSH_WORDS);
    let reviewer = Reviewer::clone_of(&repo);
    let session = Session { repo: &repo, cwd: reviewer.path(), id: READER };
    session.hook(PRE, PULL);
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    Git::run(repo.clone.path(), &PUSH_WORDS);
    FailingHead::set(&repo, repo.clone.path());
    Git::run(reviewer.path(), &PULL_WORDS);
    session.hook(POST, PULL);
    assert!(!session.hook(STOP, PULL).contains(BLOCK));
}
