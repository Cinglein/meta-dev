use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

mod branched;
pub use branched::Branched;

#[derive(Debug)]
pub struct Fixture {
    _directory: TempDir,
    work: PathBuf,
}

impl Fixture {
    const WORKING: &str =
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n[maintenance]\n\tauto = false\n";
    const BARE: &str =
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = true\n[maintenance]\n\tauto = false\n";
    const NAME: &str = "fixture";
    const EMAIL: &str = "fixture@invalid";
    const MESSAGE: &str = "fixture\n";
    const CHANGE: &str = "refs/heads/change";
    const HEAD: &str = "HEAD";
    const EVERY: &str = "*";
    const GIT_DIR: &str = ".git";
    const LINE_END: &str = "\n";
    const SEEDED: Stamp = Stamp(1_700_000_000);

    pub fn new() -> Fixture {
        let directory = TempDir::new().expect("temp dir");
        let work = directory.path().join("work");
        Self::laid(&work.join(".git"), Self::WORKING);
        Fixture { _directory: directory, work }
    }

    fn laid(git_dir: &Path, config: &str) {
        fs::create_dir_all(git_dir.join("objects/pack")).expect("objects dir");
        fs::create_dir_all(git_dir.join("refs/heads")).expect("refs dir");
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/change\n").expect("write HEAD");
        fs::write(git_dir.join("config"), config).expect("write config");
    }

    pub fn branch(&self) {
        self.committed();
        Branched::Tracked.applied(&self.work);
    }

    pub fn imported(&self, branched: Branched) -> &Fixture {
        let repository = self.opened();
        let tree = Self::tree(&repository, &self.work, Path::new(""));
        let tree = tree.map_or_else(|| repository.treebuilder(None).and_then(|empty| empty.write()), Ok);
        let tree = repository.find_tree(tree.expect("write tree")).expect("find tree");
        let parent = repository.find_reference(Self::CHANGE).and_then(|tip| tip.peel_to_commit()).ok();
        let time = git2::Time::new(Self::SEEDED.0, 0);
        let signature = git2::Signature::new(Self::NAME, Self::EMAIL, &time).expect("signature");
        let parents = Vec::from_iter(&parent);
        repository.commit(Some(Self::CHANGE), &signature, &signature, Self::MESSAGE, &tree, &parents).expect("commit");
        branched.applied(&self.work);
        self
    }

    fn tree(repository: &git2::Repository, root: &Path, under: &Path) -> Option<git2::Oid> {
        let entries = fs::read_dir(root.join(under)).expect("read dir");
        let named = entries.map(|entry| entry.expect("dir entry").file_name());
        let mut tree = repository.treebuilder(None).expect("tree builder");
        let inserted = named
            .filter(|name| under.join(name) != Path::new(Self::GIT_DIR))
            .try_for_each(|name| Self::inserted(repository, root, &under.join(name), &mut tree));
        inserted.expect("insert entries");
        (!tree.is_empty()).then(|| tree.write().expect("write tree"))
    }

    fn inserted(
        repository: &git2::Repository,
        root: &Path,
        path: &Path,
        tree: &mut git2::TreeBuilder<'_>,
    ) -> Result<(), git2::Error> {
        let name = path.file_name().expect("file name");
        if root.join(path).is_dir() {
            let subtree = Self::tree(repository, root, path);
            subtree.map_or(Ok(()), |oid| tree.insert(name, oid, git2::FileMode::Tree.into()).map(drop))
        } else {
            let blob = repository.blob(&fs::read(root.join(path)).expect("read file"))?;
            tree.insert(name, blob, git2::FileMode::Blob.into()).map(drop)
        }
    }

    pub fn committed(&self) -> &Fixture {
        let repository = self.opened();
        let mut index = repository.index().expect("index");
        index.add_all([Self::EVERY], git2::IndexAddOption::DEFAULT, None).expect("stage files");
        index.update_all([Self::EVERY], None).expect("stage removals");
        index.write().expect("write index");
        let tree = repository.find_tree(index.write_tree().expect("write tree")).expect("find tree");
        let parent = repository.head().and_then(|head| head.peel_to_commit()).ok();
        let signature = git2::Signature::now(Self::NAME, Self::EMAIL).expect("signature");
        let parents = Vec::from_iter(&parent);
        repository.commit(Some(Self::HEAD), &signature, &signature, Self::MESSAGE, &tree, &parents).expect("commit");
        self
    }

    pub fn shared_origin(&self, other: &Fixture) -> PathBuf {
        let bare = self.work.with_file_name("origin.git");
        Self::laid(&bare, Self::BARE);
        self.pointed(&bare);
        other.pointed(&bare);
        bare
    }

    pub fn pointed(&self, origin: &Path) -> &Fixture {
        let remote = format!("[remote \"origin\"]\n\turl = {}\n", origin.display());
        fs::write(self.work.join(".git/config"), [Self::WORKING, &remote].concat()).expect("write config");
        self
    }

    pub fn beside(&self) -> PathBuf {
        self.work.with_file_name("beside")
    }

    pub fn revision(&self, name: &str) -> String {
        let tip = fs::read_to_string(self.work.join(".git/refs/heads/change")).ok().filter(|_| name == "HEAD");
        tip.unwrap_or_else(|| {
            let found = self.opened().revparse_single(name).map(|found| found.id());
            found.map(|id| [id.to_string(), String::from(Self::LINE_END)].concat()).unwrap_or_default()
        })
    }

    pub fn write(&self, path: &str, text: &str) -> &Fixture {
        let file = self.path(path);
        fs::create_dir_all(file.parent().expect("parent")).expect("create dirs");
        fs::write(file, text).expect("write file");
        self
    }

    pub fn path(&self, path: &str) -> PathBuf {
        self.work.join(path)
    }

    pub fn root(&self) -> &Path {
        &self.work
    }

    fn opened(&self) -> git2::Repository {
        git2::Repository::open(&self.work).expect("open repository")
    }
}

struct Stamp(i64);
