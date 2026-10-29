use std::path::Path;

use super::{BLOCK, FAILURE, Git, NEXT_COMMIT, POST, PRE, PUSHED, Planted, Repo, STOP, Session};

const WRITE: &str = "git commit -m x";
const CD_FIRST: &str = "cd quiet && git commit -m x";
const READ: &str = "cargo test";
const BLIND: &str = "BLIND7";
const LOCKED: &str = "LOCKED8";
const LATER: &str = "3999999999";
const NO_REFLOG: [&str; 4] = ["-c", "core.logAllRefUpdates=false", "-b", "feature"];
const PUSH_ASIDE: [&str; 4] = ["push", "-q", "origin", "HEAD:aside"];
const REFLOG_OFF: &str = "reflog";
const NOT_WATCHED: &str = "CI not watched";
const UNWRITABLE: &str = "CI state unwritable: ";
const STATE_ROOT: &str = ".git/meta-ci";
const CONFIG_FILE: &str = "meta.toml";

struct Clones;

impl Clones {
    fn made_with(repo: &Repo, options: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let origin = repo.origin.to_str().unwrap();
        assert!(
            Git::run(dir.path(), &[["clone", "--shared"].as_slice(), options, &[origin, "."]].concat())
                .status
                .success()
        );
        std::fs::copy(repo.clone.path().join(CONFIG_FILE), dir.path().join(CONFIG_FILE)).unwrap();
        dir
    }
}

#[test]
fn a_reflog_that_is_off_or_an_unwritable_state_dir_fails_closed() {
    let repo = Repo::new(FAILURE);
    let clone: &Path = repo.clone.path();
    Git::pushed(&repo, PUSHED);
    let quiet = Clones::made_with(&repo, &NO_REFLOG);
    let away = quiet.path().parent().unwrap();
    let [cd_first, blind, locked] =
        [(away, BLIND), (quiet.path(), BLIND), (clone, LOCKED)].map(|(cwd, id)| Session { repo: &repo, cwd, id });
    cd_first.hook(PRE, CD_FIRST);
    Git::run(quiet.path(), &NEXT_COMMIT);
    Git::run(quiet.path(), &PUSH_ASIDE);
    let off = blind.hook(STOP, READ);
    std::fs::create_dir_all(clone.join(STATE_ROOT)).unwrap();
    let open = Planted::locked(&clone.join(STATE_ROOT));
    locked.hook(PRE, WRITE);
    Git::run_at(clone, &NEXT_COMMIT, LATER);
    let warned = locked.hook(POST, WRITE);
    let stop = locked.hook(STOP, READ);
    std::fs::set_permissions(clone.join(STATE_ROOT), open).unwrap();
    assert!(
        off.contains(REFLOG_OFF) && warned.contains(NOT_WATCHED) && stop.contains(BLOCK) && stop.contains(UNWRITABLE)
    );
}
