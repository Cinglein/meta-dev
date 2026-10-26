use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const META: &str = env!("CARGO_BIN_EXE_meta");
const IDENTITY: [&str; 6] =
    ["-c", "user.name=test", "-c", "user.email=test@meta.invalid", "-c", "init.defaultBranch=main"];
pub const CONFIG: &str = "remote = \"origin\"\n";
const EXCLUDED: &str = "target\njunk\n";
pub const CARGO_TAG: &str =
    "Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag created by cargo.\n";
const CARGO_LOCK: &str = "debug/.cargo-lock";
pub const ORIGIN: &str = "origin";
const PROJECT: &str = "project";
pub const COMMIT: [&str; 5] = ["commit", "-q", "--allow-empty", "-m", "work"];
const NOT_FOUND: &str = "not found";

#[derive(Debug, PartialEq, Eq)]
pub struct Transcript(String);

pub struct Run(Output);

impl Run {
    pub fn transcript(&self) -> String {
        [String::from_utf8_lossy(&self.0.stdout), String::from_utf8_lossy(&self.0.stderr)].concat()
    }

    fn refuses(&self, path: &Path, reason: &str) -> bool {
        self.transcript()
            .lines()
            .any(|line| path.to_str().is_some_and(|path| line.contains(path)) && line.ends_with(reason))
    }
}

pub struct Git;

impl Git {
    pub fn run(dir: &Path, words: &[&str]) -> Run {
        Run(Command::new("git").arg("-C").arg(dir).args(IDENTITY).args(words).output().unwrap())
    }

    pub fn worktree(repo: &Path, worktree: PathBuf) -> PathBuf {
        Self::run(repo, &["worktree", "add", "--detach", worktree.to_str().unwrap()]);
        worktree
    }
}

pub struct CacheTag;

impl CacheTag {
    pub fn plant(dir: PathBuf, tag: &str) -> PathBuf {
        std::fs::create_dir_all(dir.join(CARGO_LOCK).parent().unwrap()).unwrap();
        std::fs::write(dir.join(CARGO_LOCK), "").unwrap();
        std::fs::write(dir.join("CACHEDIR.TAG"), tag).unwrap();
        dir
    }
}

pub struct Scene {
    root: tempfile::TempDir,
}

impl Scene {
    pub fn new() -> Self {
        let scene = Self { root: tempfile::tempdir().unwrap() };
        Git::run(scene.root.path(), &["init", "-q", "--bare", ORIGIN]);
        Git::run(scene.root.path(), &["clone", "-q", ORIGIN, PROJECT]);
        std::fs::write(scene.project().join("meta.toml"), CONFIG).unwrap();
        std::fs::write(scene.project().join(".git/info/exclude"), EXCLUDED).unwrap();
        Git::run(&scene.project(), &["add", "meta.toml"]);
        Git::run(&scene.project(), &["commit", "-q", "-m", "start"]);
        Git::run(&scene.project(), &["push", "-q", ORIGIN, "HEAD:main"]);
        scene
    }

    pub fn place(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    pub fn project(&self) -> PathBuf {
        self.place(PROJECT)
    }

    pub fn cloned(&self, source: &str, name: &str) -> PathBuf {
        Git::run(self.root.path(), &["clone", "-q", "--shared", source, name]);
        self.place(name)
    }

    pub fn meta_rm(&self, paths: &[PathBuf]) -> Run {
        let mut command = Command::new(META);
        command.current_dir(self.project()).arg("rm").args(paths).env("META_ACTOR", "rm-test");
        Run(command.output().unwrap())
    }

    pub fn keeps(&self, refusals: &[Refusal]) -> Result<(), Transcript> {
        let kept: Vec<PathBuf> = refusals.iter().map(|refusal| self.place(refusal.name)).collect();
        let run = self.meta_rm(&kept);
        (refusals.iter().all(|refusal| {
            (self.place(refusal.name).exists() || refusal.reason == NOT_FOUND)
                && run.refuses(&self.place(refusal.name), refusal.reason)
        }) && !run.0.status.success())
        .then_some(())
        .ok_or_else(|| Transcript(run.transcript()))
    }
}

pub struct Refusal {
    name: &'static str,
    reason: &'static str,
}

impl Refusal {
    pub const fn new(name: &'static str, reason: &'static str) -> Self {
        Self { name, reason }
    }
}
