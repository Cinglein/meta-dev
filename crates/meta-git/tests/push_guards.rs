#![allow(clippy::unwrap_used)]

use meta_git::{FastForward, RemoteChange};

const BRANCH: &str = "refs/heads/meta/log";
const PACKS: &str = "objects/pack";
const GARBAGE: &str = "not an object id\n";
const UNREADABLE: &str = "corrupted";
const NAME: &str = "agent";

#[test]
fn a_repeated_push_writes_no_pack_and_an_unreadable_remote_ref_refuses_with_its_reading_error() {
    let root = tempfile::tempdir().unwrap();
    let here = git2::Repository::init(root.path().join(NAME)).unwrap();
    let there = git2::Repository::init_bare(root.path().join(BRANCH)).unwrap();
    let tree = here.find_tree(here.treebuilder(None).unwrap().write().unwrap()).unwrap();
    let signature = git2::Signature::now(NAME, NAME).unwrap();
    let commit = here.commit(None, &signature, &signature, NAME, &tree, &[]).unwrap();
    let push = FastForward::new(commit.into(), BRANCH.parse().unwrap());
    push.moved(&here, &there).unwrap();
    let packs = || std::fs::read_dir(there.path().join(PACKS)).unwrap().count();
    let before = packs();
    push.moved(&here, &there).unwrap();
    assert_eq!(packs(), before);
    std::fs::write(there.path().join(BRANCH), GARBAGE).unwrap();
    let refused = push.moved(&here, &there).unwrap_err();
    assert!(refused.message().contains(UNREADABLE), "{}", refused.message());
}
