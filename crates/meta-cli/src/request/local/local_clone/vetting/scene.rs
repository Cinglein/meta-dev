use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::context::Context;
use crate::request::local::recorded_worktree::RecordedWorktree;
use crate::request::local::worktree_id::WorktreeId;

// The one repo fixture for gc: an origin and a clone seeded by one fast-import stream each, and one
// linked worktree with a cargo target dir. Real git, laid out as <root>/{origin,clone,trees}, so a
// test can name places as "/clone/..." and "/trees/..." and translate them to real paths.
#[derive(Debug)]
pub struct Scene {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Scene {
    pub const TASK: &'static str = "task-00000000000000b1";
    pub const PATH: &'static str = "/trees/task-00000000000000b1";
    const VIRTUAL: [&'static str; 3] = ["/clone", "/trees", "/elsewhere"];
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
    const SEED: &'static str = "commit refs/heads/main
mark :1
committer t <t@meta.invalid> 1700000000 +0000
data 4
seed
M 100644 inline meta.toml
data <<END
remote = \"origin\"
END

reset refs/heads/task-00000000000000b1
from :1

";
    const TRACKING: &'static str = "reset refs/remotes/origin/main
from :1

reset refs/remotes/origin/task-00000000000000b1
from :1

commit refs/hidden/wip
committer t <t@meta.invalid> 1700000001 +0000
data 4
wip
from :1

";
    pub const CARGO_TAG: &'static str =
        "Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag created by cargo.\n";

    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(dir.path()).unwrap();
        let scene = Self { _dir: dir, root };
        let [origin, clone] = [scene.real("/origin"), scene.real("/clone")];
        Self::laid(&origin, &origin);
        Self::laid(&clone, &clone.join(".git"));
        let remote = format!("[remote \"origin\"]\n\turl = {}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n", origin.display());
        let config = clone.join(".git/config");
        std::fs::write(&config, [std::fs::read_to_string(&config).unwrap(), remote].concat()).unwrap();
        std::fs::create_dir_all(clone.join(".git/info")).unwrap();
        std::fs::write(clone.join(".git/info/exclude"), "target/\n").unwrap();
        std::thread::scope(|scope| {
            scope.spawn(|| Self::imported(&origin, Self::SEED));
            Self::imported(&clone, &[Self::SEED, Self::TRACKING].concat());
        });
        let path = scene.real(Self::PATH);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        scene.git(&clone, &["worktree", "add", "-q", path.to_str().unwrap(), Self::TASK]);
        std::fs::create_dir_all(path.join("target/debug")).unwrap();
        std::fs::write(path.join("target/debug/.cargo-lock"), "").unwrap();
        std::fs::write(path.join("target/CACHEDIR.TAG"), Self::CARGO_TAG).unwrap();
        scene
    }

    pub fn real(&self, place: &str) -> PathBuf {
        self.root.join(place.trim_start_matches('/'))
    }

    // Replace each virtual place in a text with its real path.
    pub fn translated(&self, text: &str) -> String {
        Self::VIRTUAL.iter().fold(text.to_owned(), |text, place| {
            text.replace(place, self.real(place).to_str().unwrap_or_default())
        })
    }

    // Replace the real root with nothing, so output reads as virtual places again.
    pub fn shown(&self, text: &str) -> String {
        text.replace(self.root.to_str().unwrap_or_default(), "")
    }

    pub fn context(&self) -> Context {
        let [checkout, common, root] = ["/clone", "/clone/.git", "/trees"].map(|place| self.real(place));
        Context::new(checkout, common, root, "origin".parse().unwrap())
    }

    pub fn worktree(&self) -> RecordedWorktree {
        self.recorded(Self::PATH)
    }

    pub fn recorded(&self, place: &str) -> RecordedWorktree {
        let identity = WorktreeId::parsed(Path::new(Self::TASK)).unwrap();
        RecordedWorktree::new(self.real(place), Self::TASK.parse().unwrap()).identified(identity)
    }

    pub fn git(&self, dir: &Path, words: &[&str]) -> String {
        let output = Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap();
        assert!(output.status.success(), "{words:?}: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    fn laid(dir: &Path, git_dir: &Path) {
        std::fs::create_dir_all(git_dir.join("objects")).unwrap();
        std::fs::create_dir_all(git_dir.join("refs")).unwrap();
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        let bare = dir == git_dir;
        let config = format!("[core]\n\trepositoryformatversion = 0\n\tbare = {bare}\n[maintenance]\n\tauto = false\n");
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
