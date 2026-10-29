#![allow(clippy::unwrap_used)]

use meta_git::{
    Addition, FileCommit, FileLine, Files, FilesAt, FirstAddition, FirstParents, GitFailure, Identity, Layout,
    ObjectId, Reads, RefMove, RefName, RefTarget, Repository, Writes,
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

struct Merges(git2::Repository);

impl Merges {
    const FILE: &'static str = "f";
    const BASE: &'static str = "x\n";
    const LEFT: &'static str = concat!("x\nL /tmp/", "claude-1\n");
    const RIGHT: &'static str = "x\nR\n";
    const MERGED: [&'static str; 2] =
        [concat!("x\nL /tmp/", "claude-1\nR\n"), concat!("x\nL /tmp/", "claude-1\nR\n/tmp/", "claude-2\n")];
    const PUSHED: &'static str = "refs/remotes/origin/main";
    const NEEDLE: &'static str = concat!("/tmp/", "claude-");

    fn commit(&self, text: &str, parents: &[git2::Oid], head: bool) -> git2::Oid {
        let mut tree = self.0.treebuilder(None).unwrap();
        tree.insert(Self::FILE, self.0.blob(text.as_bytes()).unwrap(), git2::FileMode::Blob.into()).unwrap();
        let tree = self.0.find_tree(tree.write().unwrap()).unwrap();
        let parents: Vec<git2::Commit<'_>> =
            parents.iter().map(|parent| self.0.find_commit(*parent).unwrap()).collect();
        let signature = git2::Signature::now(Self::FILE, Self::FILE).unwrap();
        let parents: Vec<&git2::Commit<'_>> = parents.iter().collect();
        self.0.commit(head.then_some("HEAD"), &signature, &signature, text, &tree, &parents).unwrap()
    }

    fn first(merged: &str) -> Option<bool> {
        let directory = tempfile::tempdir().unwrap();
        let scene = Merges(git2::Repository::init_bare(directory.path()).unwrap());
        let base = scene.commit(Self::BASE, &[], false);
        let left = scene.commit(Self::LEFT, &[base], false);
        scene.0.reference(Self::PUSHED, left, true, Self::FILE).unwrap();
        let right = scene.commit(Self::RIGHT, &[base], false);
        let merge = scene.commit(merged, &[left, right], true);
        let short = scene.0.find_commit(merge).unwrap().as_object().short_id().unwrap().as_str().unwrap().to_owned();
        let leak = FirstAddition::new("origin".parse().unwrap(), Self::NEEDLE.to_owned());
        let found = Repository::open(directory.path()).unwrap().read(&leak).unwrap();
        let last = FileLine::from(u32::try_from(merged.lines().count()).unwrap());
        found.map(|found| found.commit() == short && found == Addition::new(Self::FILE.to_owned(), last, short))
    }
}

#[test]
fn a_merge_adds_only_lines_new_to_every_parent_and_pushed_commits_are_skipped() {
    assert_eq!(Merges::MERGED.map(Merges::first), [None, Some(true)]);
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

#[test]
fn discovery_finds_nothing_outside_a_repository_and_fails_on_a_broken_link() {
    let directory = tempfile::tempdir().unwrap();
    assert!(matches!(Repository::discovered(directory.path()), Ok(None)));
    std::fs::write(directory.path().join(".git"), FILE).unwrap();
    assert!(Repository::discovered(directory.path()).is_err());
}
