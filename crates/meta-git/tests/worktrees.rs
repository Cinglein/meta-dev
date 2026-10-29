#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use meta_git::{
    BranchCreate, BranchDelete, ConfigFlag, FileCommit, Files, Identity, Ignored, Layout, ObjectId, Reads, RefMove,
    Repository, Revision, Setting, WorktreeAdd, WorktreePrune, WorktreeRemove, Writes,
};

const FILE: &str = "tracked.txt";
const MAIN: &str = "refs/heads/main";
const CHECKOUT: &str = "checkout";
const KEPT: &str = "kept";
const GONE: &str = "gone";
const NAME: &str = "agent";
const EMAIL: &str = "1+agent@users.noreply.github.com";
const WORKTREE_CONFIG: &str = "extensions.worktreeConfig";
const PINNED: [&str; 3] = ["user.email", "author.email", "committer.email"];
const EXCLUDE: &str = ".git/info/exclude";
const IGNORED: &str = "local.toml";
const LINE_END: &str = "\n";
const ADMIN: &str = ".git/worktrees";

struct Scene {
    repository: Repository,
    tip: ObjectId,
}

impl Scene {
    fn seeded(root: &Path) -> Scene {
        let repository = Repository::created(&root.join(CHECKOUT), Layout::Working).unwrap();
        let files: Files = [(FILE.to_owned(), Some(FILE.to_owned()))].into_iter().collect();
        let author = Identity::new(NAME.to_owned(), EMAIL.to_owned());
        let tip = repository.write(&FileCommit::new(None, files, author, FILE.to_owned())).unwrap();
        repository.write(&RefMove::new(MAIN.parse().unwrap(), None, tip)).unwrap();
        repository.write(&ConfigFlag::from(WORKTREE_CONFIG.to_owned())).unwrap();
        Scene { repository, tip }
    }

    fn added(&self, root: &Path, name: &str) {
        self.repository.write(&BranchCreate::new(name.to_owned(), self.tip)).unwrap();
        let identity = Identity::new(NAME.to_owned(), EMAIL.to_owned());
        self.repository.write(&WorktreeAdd::new(name.to_owned(), root.join(name), name.to_owned(), identity)).unwrap();
    }

    fn excluded(&self, root: &Path) -> bool {
        std::fs::write(root.join(CHECKOUT).join(EXCLUDE), [IGNORED, LINE_END].concat()).unwrap();
        let ignored = |name: &str| self.repository.read(&Ignored::from(PathBuf::from(name))).unwrap();
        ignored(IGNORED) && !ignored(FILE)
    }

    fn pinned(worktree: &Path) -> Vec<Option<String>> {
        let opened = Repository::open(worktree).unwrap();
        PINNED.iter().map(|key| opened.read(&Setting::from((*key).to_owned())).unwrap()).collect()
    }
}

#[test]
fn an_added_worktree_is_checked_out_and_pinned_a_gone_one_is_pruned_and_a_removed_one_leaves_no_branch() {
    let root = tempfile::tempdir().unwrap();
    let scene = Scene::seeded(root.path());
    let Scene { repository, tip } = &scene;
    let read = |spec: &str| repository.read(&Revision::from(spec.to_owned())).unwrap();
    scene.added(root.path(), KEPT);
    scene.added(root.path(), GONE);
    std::fs::remove_dir_all(root.path().join(GONE)).unwrap();
    repository.write(&WorktreePrune).unwrap();
    let admin = root.path().join(CHECKOUT).join(ADMIN);
    assert!(
        root.path().join(KEPT).join(FILE).exists()
            && admin.join(KEPT).exists()
            && !admin.join(GONE).exists()
            && read(MAIN) == Some(*tip)
    );
    assert!(Scene::pinned(&root.path().join(KEPT)).iter().all(|email| email.as_deref() == Some(EMAIL)));
    repository.write(&WorktreeRemove::from(KEPT.to_owned())).unwrap();
    repository.write(&BranchDelete::from(KEPT.to_owned())).unwrap();
    assert!(!root.path().join(KEPT).exists() && read(KEPT).is_none() && scene.excluded(root.path()));
}
