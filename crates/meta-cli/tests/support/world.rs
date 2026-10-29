#![allow(dead_code, clippy::unwrap_used)]

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

pub struct World {
    pub origin: tempfile::TempDir,
    pub clone: tempfile::TempDir,
    pub trees: tempfile::TempDir,
}

impl World {
    const IDENTITY: [&'static str; 8] = [
        "-c",
        "user.name=test",
        "-c",
        "user.email=test@meta.invalid",
        "-c",
        "init.defaultBranch=main",
        "-c",
        "maintenance.auto=false",
    ];
    const BARE: &'static str = "[core]\n\trepositoryformatversion = 0\n\tbare = true\n[maintenance]\n\tauto = false\n";
    const WORKING: &'static str =
        "[core]\n\trepositoryformatversion = 0\n\tbare = false\n[maintenance]\n\tauto = false\n";
    const HEAD: &'static str = "ref: refs/heads/main\n";
    pub const CONFIG: &'static str = "remote = \"origin\"\n";
    const SEED: &'static str = "commit refs/heads/main
committer t <t@meta.invalid> 1700000000 +0000
data 4
seed
M 100644 inline meta.toml
data <<END
remote = \"origin\"
END

";

    pub fn new(branches: &[&str], origin_extra: &str) -> Self {
        let world = Self {
            origin: tempfile::tempdir().unwrap(),
            clone: tempfile::tempdir().unwrap(),
            trees: tempfile::tempdir().unwrap(),
        };
        Self::laid(world.origin.path(), Self::BARE);
        Self::laid(&world.clone.path().join(".git"), Self::WORKING);
        let remote = format!(
            "[remote \"origin\"]\n\turl = {}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n",
            world.origin.path().display()
        );
        let config = world.clone.path().join(".git/config");
        std::fs::write(&config, [std::fs::read_to_string(&config).unwrap(), remote].concat()).unwrap();
        std::fs::write(world.clone.path().join("meta.toml"), Self::CONFIG).unwrap();
        let reset = |prefix: &str, names: &[&str]| -> String {
            names.iter().flat_map(|name| ["reset ", prefix, name, "\nfrom refs/heads/main\n\n"]).collect()
        };
        let heads = reset("refs/heads/", branches);
        let tracking = reset("refs/remotes/origin/", &[["main"].as_slice(), branches].concat());
        std::thread::scope(|scope| {
            scope.spawn(|| Self::imported(world.origin.path(), &[Self::SEED, &heads, origin_extra].concat()));
            Self::imported(world.clone.path(), &[Self::SEED, &heads, &tracking].concat());
        });
        world
    }

    pub fn added(&self, branches: &[&str]) -> &Self {
        let trees = self.trees();
        let added: Vec<Output> = branches
            .iter()
            .map(|task| self.git(&["worktree", "add", "-q", trees.join(task).to_str().unwrap(), task]))
            .collect();
        assert!(added.iter().all(|output| output.status.success()), "{added:?}");
        self
    }

    pub fn git(&self, words: &[&str]) -> Output {
        Self::run(self.clone.path(), words)
    }

    pub fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }

    pub fn trees(&self) -> PathBuf {
        std::fs::canonicalize(self.trees.path()).unwrap()
    }

    pub fn common(&self) -> PathBuf {
        std::fs::canonicalize(self.clone.path()).unwrap().join(".git")
    }

    pub fn checkout(&self) -> PathBuf {
        std::fs::canonicalize(self.clone.path()).unwrap()
    }

    fn laid(git_dir: &Path, config: &str) {
        std::fs::create_dir_all(git_dir.join("objects")).unwrap();
        std::fs::create_dir_all(git_dir.join("refs")).unwrap();
        std::fs::write(git_dir.join("HEAD"), Self::HEAD).unwrap();
        std::fs::write(git_dir.join("config"), config).unwrap();
    }

    fn imported(repo: &Path, stream: &str) {
        let mut command = Command::new("git");
        let piped = command.arg("-C").arg(repo).args(["fast-import", "--quiet"]).stdin(Stdio::piped());
        let mut child = piped.spawn().unwrap();
        child.stdin.take().unwrap().write_all(stream.as_bytes()).unwrap();
        assert!(child.wait().unwrap().success());
    }
}
