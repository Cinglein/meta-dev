#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use meta_git::{
    Head, ObjectId, Reads, RelativeDiff, Repository, ShortHead, WorktreeDetach, WorktreePaths, WorktreeRemove, Writes,
};

const MAIN: &str = "main";
const LINKED: [&str; 2] = [".tmpgate", ".tmpother"];
const DETACHED: &str = "detached-tmpgate";
const ADMIN: &str = "worktrees/detached-tmpgate";
const UP: &str = "..";
const OUTSIDE: &str = "top.rs";
const SOURCE: &str = "consumer/a.rs";
const BASE: &str = "refs/remotes/origin/main";
const PATCH: &str = "diff --git a/a.rs b/a.rs\nindex ee2363a..5c80f32 100644\n--- a/a.rs\n+++ b/a.rs\n@@ -1 +1 @@\n-before\n\\ No newline at end of file\n+after\n\\ No newline at end of file\n";
const TEXTS: [&str; 2] = ["before", "after"];
const SHORT_LEN: usize = 7;

struct Main {
    root: tempfile::TempDir,
    commit: git2::Oid,
}

impl Main {
    fn made() -> Main {
        let root = tempfile::tempdir().unwrap();
        let main = root.path().join(MAIN);
        let source = main.join(SOURCE);
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, TEXTS[0]).unwrap();
        std::fs::write(main.join(OUTSIDE), TEXTS[0]).unwrap();
        let repository = git2::Repository::init(&main).unwrap();
        let mut index = repository.index().unwrap();
        index.add_path(Path::new(SOURCE)).unwrap();
        index.add_path(Path::new(OUTSIDE)).unwrap();
        index.write().unwrap();
        let tree = repository.find_tree(index.write_tree().unwrap()).unwrap();
        let signature = git2::Signature::now(MAIN, MAIN).unwrap();
        let commit = repository.commit(Some("HEAD"), &signature, &signature, MAIN, &tree, &[]).unwrap();
        repository.reference(BASE, commit, true, BASE).unwrap();
        Main { root, commit }
    }

    fn opened(&self) -> Repository {
        Repository::open(&self.root.path().join(MAIN)).unwrap()
    }

    fn paths(&self) -> Vec<PathBuf> {
        let listed = self.opened().read(&WorktreePaths).unwrap();
        listed.iter().map(|path| path.canonicalize().unwrap_or_else(|_| path.clone())).collect()
    }
}

#[test]
fn a_detached_worktree_is_named_after_its_path_leaves_no_branch_and_only_it_is_removed_by_its_path() {
    let main = Main::made();
    let [linked, other] = LINKED.map(|name| main.root.path().join(name));
    let detached = |path: &PathBuf| {
        std::fs::create_dir(path).unwrap();
        main.opened().write(&WorktreeDetach::new(path.clone(), ObjectId::from(main.commit)))
    };
    [&linked, &other].into_iter().try_for_each(detached).unwrap();
    let head = Repository::open(&linked).unwrap().read(&Head).unwrap();
    let canonical = |path: &Path| path.canonicalize().unwrap();
    let opened = git2::Repository::open(main.root.path().join(MAIN)).unwrap();
    let branch = opened.find_branch(DETACHED, git2::BranchType::Local).ok().map(drop);
    let named = opened.path().join(ADMIN).exists();
    assert!(named && branch.is_none() && head.commit() == Some(ObjectId::from(main.commit)) && head.branch().is_none());
    main.opened().write(&WorktreeRemove::At(linked.join(UP).join(LINKED[1]))).unwrap();
    assert!(main.paths() == [canonical(&main.root.path().join(MAIN)), canonical(&linked)] && !other.exists());
    main.opened().write(&WorktreeRemove::At(linked.clone())).unwrap();
    assert!(main.paths() == [canonical(&main.root.path().join(MAIN))] && !linked.exists());
}

#[test]
fn the_short_head_and_the_relative_diff_read_like_git_rev_parse_short_and_diff_relative_merge_base() {
    let main = Main::made();
    std::fs::write(main.root.path().join(MAIN).join(SOURCE), TEXTS[1]).unwrap();
    std::fs::write(main.root.path().join(MAIN).join(OUTSIDE), TEXTS[1]).unwrap();
    let below = main.root.path().join(MAIN).join(Path::new(SOURCE).parent().unwrap());
    let diff = main.opened().read(&RelativeDiff::new(BASE.to_owned(), below)).unwrap();
    let short = main.opened().read(&ShortHead).unwrap();
    assert_eq!((short.len(), main.commit.to_string().starts_with(&short)), (SHORT_LEN, true));
    assert_eq!(diff, PATCH);
}
