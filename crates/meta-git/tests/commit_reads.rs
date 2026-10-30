#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use meta_git::{CommitEmails, MergeBase, ObjectId, Reads, RefName, RemoteBranches, Repository, TreeText};

const FILES: [(&str, &str); 2] = [("top", "1"), ("dir/leaf", "l")];
const DIRECTORY: &str = "dir";
const ABSENT: &str = "absent";
const MISSING: &str = "1234567890123456789012345678901234567890";
const PAIR_LEN: usize = 2;
const PEOPLE: [[&str; PAIR_LEN]; 3] = [["a1@x", "c1@x"], ["a2@x", "c2@x"], ["a3@x", "c3@x"]];
const EMAILS: [&str; 4] = ["a3@x", "c3@x", "a2@x", "c2@x"];
const REMOTES: [&str; 4] =
    ["refs/remotes/origin/HEAD", "refs/remotes/origin/main", "refs/remotes/origin/old", "refs/remotes/origin/side"];
const SIDE: &str = "refs/remotes/origin/side";
const OLD: &str = "refs/remotes/origin/old";
const BRANCH: &str = "refs/heads/main";

struct History {
    repository: Repository,
    first: ObjectId,
    second: ObjectId,
    third: ObjectId,
}

impl History {
    fn made(root: &Path) -> History {
        let repository = git2::Repository::init(root).unwrap();
        FILES
            .iter()
            .try_for_each(|(name, text)| {
                std::fs::create_dir_all(root.join(name).parent().unwrap())?;
                std::fs::write(root.join(name), text)
            })
            .unwrap();
        let mut index = repository.index().unwrap();
        index.add_all(["*"], git2::IndexAddOption::DEFAULT, None).unwrap();
        let snapshot = repository.find_tree(index.write_tree().unwrap()).unwrap();
        let commit = |parent: Option<git2::Oid>, [author, committer]: [&str; PAIR_LEN]| {
            let [author, committer] = [author, committer].map(|email| git2::Signature::now(email, email).unwrap());
            let parents: Vec<_> = parent.iter().map(|parent| repository.find_commit(*parent).unwrap()).collect();
            repository
                .commit(
                    None,
                    &author,
                    &committer,
                    author.email().unwrap(),
                    &snapshot,
                    &parents.iter().collect::<Vec<_>>(),
                )
                .unwrap()
        };
        let [one, two, three] = PEOPLE;
        let first = commit(None, one);
        let second = commit(Some(first), two);
        let third = commit(Some(second), three);
        let side = commit(Some(first), three);
        let [head, main, old, side_ref] = REMOTES;
        [(main, second), (old, first), (side_ref, side), (BRANCH, third)]
            .iter()
            .try_for_each(|(name, id)| repository.reference(name, *id, true, name).map(drop))
            .unwrap();
        repository.reference_symbolic(head, main, true, head).unwrap();
        repository.set_head(BRANCH).unwrap();
        History {
            repository: Repository::open(root).unwrap(),
            first: first.into(),
            second: second.into(),
            third: third.into(),
        }
    }

    fn branches(&self, asked: RemoteBranches) -> Option<Vec<RefName>> {
        let mut listed = self.repository.read(&asked).ok()?;
        listed.sort();
        Some(listed)
    }
}

#[test]
fn tree_text_reads_a_blob_like_git_cat_file_and_a_directory_or_missing_commit_fails() {
    let root = tempfile::tempdir().unwrap();
    let history = History::made(root.path());
    let text = |commit: ObjectId, path: &str| history.repository.read(&TreeText::new(commit, PathBuf::from(path))).ok();
    let read = FILES.map(|(path, _)| text(history.third, path));
    assert_eq!(read, FILES.map(|(_, text)| Some(Some(text.to_owned()))));
    assert_eq!([text(history.third, ABSENT), text(history.third, DIRECTORY)], [Some(None), None]);
    assert_eq!(text(MISSING.parse().unwrap(), FILES[0].0), None);
}

#[test]
fn merge_base_and_commit_emails_match_git_and_a_missing_base_fails() {
    let root = tempfile::tempdir().unwrap();
    let history = History::made(root.path());
    let base = |spec: &str| history.repository.read(&MergeBase::from(spec)).ok();
    let emails = |spec: &str| history.repository.read(&CommitEmails::from(spec)).ok();
    assert_eq!([base(SIDE), base(MISSING)], [Some(history.first), None]);
    assert_eq!([emails(OLD), emails(MISSING)], [Some(EMAILS.map(str::to_owned).to_vec()), None]);
}

#[test]
fn remote_branches_match_git_branch_remotes_contains_and_points_at_with_a_symbolic_head() {
    let root = tempfile::tempdir().unwrap();
    let history = History::made(root.path());
    let names = |names: &[&str]| Some(names.iter().map(|name| name.parse().unwrap()).collect::<Vec<RefName>>());
    let [head, main, old, _] = REMOTES;
    assert_eq!(history.branches(RemoteBranches::Containing(history.second)), names(&[head, main]));
    assert_eq!(history.branches(RemoteBranches::Containing(history.first)), names(&REMOTES));
    assert!(
        history.branches(RemoteBranches::At(history.first)) == names(&[old])
            && history.branches(RemoteBranches::At(MISSING.parse().unwrap())).is_none()
    );
}
