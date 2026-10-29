#![allow(clippy::unwrap_used)]

use meta_git::{
    FileCommit, Files, FilesAt, FirstParents, GitFailure, Identity, Layout, ObjectId, Reads, RefMove, RefName,
    RefTarget, Repository, Writes,
};

const FILE: &str = "events.jsonl";
const BRANCH: &str = "refs/heads/meta/log";
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
    repository.write(&RefMove::new(BRANCH.parse().unwrap(), second)).unwrap();
    let Ok(branch) = BRANCH.parse::<RefName>();
    let tip = repository.read(&RefTarget::from(branch)).unwrap().unwrap();
    let files = repository.read(&FilesAt::from(tip)).unwrap().unwrap();
    assert_eq!(files.only(FILE), Some(BRANCH));
    assert_eq!(repository.read(&FirstParents::from(tip)).unwrap(), vec![second, first]);
}

#[test]
fn a_reftable_repository_fails_closed_with_a_clear_error() {
    let directory = tempfile::tempdir().unwrap();
    Repository::created(directory.path(), Layout::Bare).unwrap();
    std::fs::write(directory.path().join("config"), REFTABLE).unwrap();
    let refused = Repository::open(directory.path());
    assert!(matches!(refused, Err(GitFailure::Reftable(path)) if path == directory.path()));
}
