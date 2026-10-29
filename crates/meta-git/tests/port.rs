#![allow(clippy::unwrap_used)]

use meta_git::{
    FileCommit, Files, FilesAt, FirstParents, GitFailure, Identity, Layout, ObjectId, Reads, RefMove, RefName,
    RefTarget, Repository, Writes,
};

const FILE: &str = "events.jsonl";
const BRANCH: &str = "refs/heads/meta/log";
const LOCK: &str = "refs/heads/meta/log.lock";
const WHOLE: &str = "0123456789abcdef0123456789abcdef01234567";
const UNHEX: &str = "g123456789abcdef0123456789abcdef01234567";
const REFTABLE: &str = "[core]\n\trepositoryformatversion = 1\n\tbare = true\n[extensions]\n\trefstorage = reftable\n";

struct Commits;

impl Commits {
    fn written(repository: &Repository, parent: Option<ObjectId>, line: &str) -> ObjectId {
        let files: Files = [(FILE.to_owned(), Some(line.to_owned()))].into_iter().collect();
        let author = Identity::new(FILE.to_owned(), FILE.to_owned());
        repository.write(&FileCommit::new(parent, files, author, line.to_owned())).unwrap()
    }
}

#[test]
fn commits_written_in_process_read_back_through_their_ref_and_first_parents() {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::created(directory.path(), Layout::Bare).unwrap();
    let first = Commits::written(&repository, None, FILE);
    let second = Commits::written(&repository, Some(first), BRANCH);
    repository.write(&RefMove::new(BRANCH.parse().unwrap(), None, second)).unwrap();
    let Ok(branch) = BRANCH.parse::<RefName>();
    let tip = repository.read(&RefTarget::from(branch)).unwrap().unwrap();
    let files = repository.read(&FilesAt::from(tip)).unwrap().unwrap();
    assert_eq!(files.only(FILE), Some(BRANCH));
    assert_eq!(repository.read(&FirstParents::from(tip)).unwrap(), vec![second, first]);
    assert_eq!(format!("{repository:?}"), format!("Repository({:?})", directory.path()));
}

#[test]
fn a_ref_move_decided_on_a_stale_target_or_meeting_a_lock_is_refused_and_leaves_the_ref() {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::created(directory.path(), Layout::Bare).unwrap();
    let first = Commits::written(&repository, None, FILE);
    let second = Commits::written(&repository, Some(first), BRANCH);
    let third = Commits::written(&repository, Some(second), WHOLE);
    let Ok(branch) = BRANCH.parse::<RefName>();
    let moved = |from: Option<ObjectId>, to: ObjectId| repository.write(&RefMove::new(branch.clone(), from, to));
    moved(None, first).unwrap();
    moved(Some(first), second).unwrap();
    let stale = [moved(Some(first), third), moved(None, third)];
    std::fs::write(directory.path().join(LOCK), FILE).unwrap();
    let refused: Vec<_> = stale.into_iter().chain([moved(Some(second), third), moved(Some(second), first)]).collect();
    assert!(refused.iter().all(|move_| matches!(move_, Err(GitFailure::RefMoved(name)) if *name == branch)));
    assert_eq!(repository.read(&RefTarget::from(branch.clone())).unwrap(), Some(second));
}

#[test]
fn only_forty_hex_digits_name_an_object() {
    let named = |text: &str| text.parse::<ObjectId>().is_ok();
    assert!(named(WHOLE) && !named(&WHOLE[1..]) && !named(UNHEX));
}

#[test]
fn a_reftable_repository_fails_closed_with_a_clear_error() {
    let directory = tempfile::tempdir().unwrap();
    Repository::created(directory.path(), Layout::Bare).unwrap();
    std::fs::write(directory.path().join("config"), REFTABLE).unwrap();
    let refused = Repository::open(directory.path());
    assert!(matches!(refused, Err(GitFailure::Reftable(path)) if path == directory.path()));
}
