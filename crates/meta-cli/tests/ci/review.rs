use std::path::Path;

use super::{BLOCK, FAILURE, Fixture, Moment, NEXT_COMMIT, POST, PRE, PUSHED, Planted, Repo, STOP, Session};

const WRITE: &str = "git commit -m x";
const CD_FIRST: &str = "cd quiet && git commit -m x";
const READ: &str = "cargo test";
const BLIND: &str = "BLIND7";
const LOCKED: &str = "LOCKED8";
const LATER: Moment = Moment::after_epoch(3_999_999_999);
const ASIDE: &str = "aside";
const REFLOG_OFF: &str = "reflog";
const NOT_WATCHED: &str = "CI not watched";
const UNWRITABLE: &str = "CI state unwritable: ";
const STATE_ROOT: &str = ".git/meta-ci";
const CONFIG_FILE: &str = "meta.toml";

struct Clones;

impl Clones {
    fn quiet(repo: &Repo) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        Fixture::working(dir.path(), &repo.origin, PUSHED).quietly_tracking(PUSHED);
        std::fs::copy(repo.clone.path().join(CONFIG_FILE), dir.path().join(CONFIG_FILE)).unwrap();
        dir
    }
}

#[test]
fn a_reflog_that_is_off_or_an_unwritable_state_dir_fails_closed() {
    let repo = Repo::new(FAILURE);
    let clone: &Path = repo.clone.path();
    repo.git().pushed(PUSHED);
    let quiet = Clones::quiet(&repo);
    let away = quiet.path().parent().unwrap();
    let [cd_first, blind, locked] =
        [(away, BLIND), (quiet.path(), BLIND), (clone, LOCKED)].map(|(cwd, id)| Session { repo: &repo, cwd, id });
    cd_first.hook(PRE, CD_FIRST);
    Fixture::opened(quiet.path()).committed(NEXT_COMMIT);
    Fixture::opened(quiet.path()).pushed(ASIDE);
    let off = blind.hook(STOP, READ);
    std::fs::create_dir_all(clone.join(STATE_ROOT)).unwrap();
    let open = Planted::locked(&clone.join(STATE_ROOT));
    locked.hook(PRE, WRITE);
    repo.git().committed_at(NEXT_COMMIT, &LATER);
    let warned = locked.hook(POST, WRITE);
    let stop = locked.hook(STOP, READ);
    std::fs::set_permissions(clone.join(STATE_ROOT), open).unwrap();
    assert!(
        off.contains(REFLOG_OFF) && warned.contains(NOT_WATCHED) && stop.contains(BLOCK) && stop.contains(UNWRITABLE)
    );
}
