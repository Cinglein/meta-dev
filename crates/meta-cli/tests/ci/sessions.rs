use std::path::Path;

use super::{BLOCK, CI_WAIT, COMMIT, FAILURE, Git, NEXT_COMMIT, POST, PRE, PUSH_WORDS, Planted, Repo, STOP};

const MINE: &str = "MINE1";
const THEIRS: &str = "THEIRS2";
const READER: &str = "READER3";
const BUILD: &str = "cargo xtask ci";
const PULL: &str = "git pull";
const CONFIG_FILE: &str = "meta.toml";
const WORKTREE: &str = "other";
const ADD_WORKTREE: [&str; 6] = ["worktree", "add", "-q", WORKTREE, "-b", WORKTREE];
const PULL_WORDS: [&str; 2] = ["pull", "-q"];
const STATE_ROOT: &str = ".git/meta-ci";

struct Reviewer;

impl Reviewer {
    fn clone_of(repo: &Repo) -> tempfile::TempDir {
        let reviewer = tempfile::tempdir().unwrap();
        let origin = repo.origin.path().to_str().unwrap();
        Git::run(reviewer.path(), &["clone", "-q", "-b", PUSH_WORDS[3], origin, "."]);
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
    repo.hook_as(clone, MINE, PRE, BUILD);
    repo.hook_as(&other, THEIRS, PRE, COMMIT);
    Git::run(&other, &NEXT_COMMIT);
    repo.fail_at(&other);
    repo.hook_as(&other, THEIRS, POST, COMMIT);
    repo.hook_as(clone, MINE, POST, BUILD);
    let mine = repo.hook_as(clone, MINE, STOP, BUILD);
    let theirs = repo.hook_as(&other, THEIRS, STOP, COMMIT);
    let open = Planted::locked(&clone.join(STATE_ROOT));
    let locked = repo.hook_as(clone, READER, STOP, BUILD);
    std::fs::set_permissions(clone.join(STATE_ROOT), open).unwrap();
    assert!(!mine.contains(BLOCK) && theirs.contains(BLOCK) && !locked.contains(BLOCK));
}

#[test]
fn a_reviewers_pull_of_a_red_branch_never_blocks() {
    let repo = Repo::new(FAILURE);
    Git::run(repo.clone.path(), &PUSH_WORDS);
    let reviewer = Reviewer::clone_of(&repo);
    repo.hook_as(reviewer.path(), READER, PRE, PULL);
    Git::run(repo.clone.path(), &NEXT_COMMIT);
    Git::run(repo.clone.path(), &PUSH_WORDS);
    repo.fail_at(repo.clone.path());
    Git::run(reviewer.path(), &PULL_WORDS);
    repo.hook_as(reviewer.path(), READER, POST, PULL);
    assert!(!repo.hook_as(reviewer.path(), READER, STOP, PULL).contains(BLOCK));
}
