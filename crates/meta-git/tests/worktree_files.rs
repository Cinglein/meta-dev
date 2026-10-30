#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use meta_git::{Head, Reads, Repository, WorktreeFiles};

const SOURCE: &str = "source";
const WORK: &str = "work";
const INNER: &str = "inner";
const SUBMODULE: &str = "sub";
const PAIR_LEN: usize = 2;
const COMMITTED: [[&str; PAIR_LEN]; 3] = [[".gitignore", "*.log\n"], ["kept", "k"], ["moved", "m"]];
const UNTRACKED: [&str; 2] = ["new/inner.txt", "debug.log"];
const RENAMED: [&str; 2] = ["moved", "renamed"];
const OTHERS: [&str; 1] = ["new/inner.txt"];
const STATUS: [&str; 4] = ["moved", "new/inner.txt", "renamed", "sub"];
const TOUCHED: [&str; 5] = ["debug.log", "moved", "new/inner.txt", "renamed", "sub"];
const DIFF: [&str; 3] = ["moved", "renamed", "sub"];
const LS_FILES: [&str; 5] = [".gitignore", ".gitmodules", "kept", "renamed", "sub"];

struct Work(PathBuf);

struct Snapshot;

impl Snapshot {
    fn committed(root: &Path, files: &[[&str; PAIR_LEN]]) -> git2::Repository {
        let repository = git2::Repository::init(root).unwrap();
        files.iter().try_for_each(|[name, text]| std::fs::write(root.join(name), text)).unwrap();
        let mut index = repository.index().unwrap();
        index.add_all(["*"], git2::IndexAddOption::DEFAULT, None).unwrap();
        index.write().unwrap();
        Self::recorded(&repository);
        repository
    }

    fn recorded(repository: &git2::Repository) {
        let tree = repository.find_tree(repository.index().unwrap().write_tree().unwrap()).unwrap();
        let signature = git2::Signature::now(INNER, INNER).unwrap();
        let parent = repository.head().ok().map(|head| head.peel_to_commit().unwrap());
        repository
            .commit(Some("HEAD"), &signature, &signature, INNER, &tree, &parent.iter().collect::<Vec<_>>())
            .unwrap();
    }
}

impl Work {
    fn made(root: &Path) -> Work {
        Snapshot::committed(&root.join(SOURCE), &[[INNER, INNER]]);
        let work = root.join(WORK);
        let repository = Snapshot::committed(&work, &COMMITTED);
        let url = root.join(SOURCE);
        let mut submodule = repository.submodule(url.to_str().unwrap(), Path::new(SUBMODULE), true).unwrap();
        submodule.clone(None).unwrap();
        submodule.add_finalize().unwrap();
        Snapshot::recorded(&repository);
        Work(work)
    }

    fn changed(self) -> Work {
        let [inside, ignored] = UNTRACKED.map(|name| self.0.join(name));
        std::fs::create_dir_all(inside.parent().unwrap()).unwrap();
        [inside, ignored, self.0.join(SUBMODULE).join(INNER)]
            .iter()
            .try_for_each(|path| std::fs::write(path, ""))
            .unwrap();
        let [from, to] = RENAMED;
        std::fs::rename(self.0.join(from), self.0.join(to)).unwrap();
        let repository = git2::Repository::open(&self.0).unwrap();
        let mut index = repository.index().unwrap();
        index.remove_path(Path::new(from)).unwrap();
        index.add_path(Path::new(to)).unwrap();
        index.write().unwrap();
        self
    }

    fn listed(&self, files: WorktreeFiles) -> Vec<PathBuf> {
        let mut listed = Repository::open(&self.0).unwrap().read(&files).unwrap();
        listed.sort();
        listed
    }
}

#[test]
fn worktree_files_list_what_git_ls_files_status_and_diff_list_with_ignored_renamed_and_submodule_paths() {
    let root = tempfile::tempdir().unwrap();
    let work = Work::made(root.path()).changed();
    let head = Repository::open(&work.0).unwrap().read(&Head).unwrap().commit().unwrap();
    let paths = |names: &[&str]| names.iter().map(PathBuf::from).collect::<Vec<_>>();
    assert_eq!(work.listed(WorktreeFiles::Untracked), paths(&OTHERS));
    assert_eq!(work.listed(WorktreeFiles::Unclean), paths(&STATUS));
    assert!(
        work.listed(WorktreeFiles::ChangedSince(head)) == paths(&DIFF)
            && work.listed(WorktreeFiles::Tracked) == paths(&LS_FILES)
    );
}

#[test]
fn touched_files_are_what_git_status_lists_with_ignored_files_too() {
    let root = tempfile::tempdir().unwrap();
    let work = Work::made(root.path()).changed();
    let touched: Vec<PathBuf> = TOUCHED.iter().map(PathBuf::from).collect();
    assert_eq!(work.listed(WorktreeFiles::Touched), touched);
}
