#![allow(clippy::unwrap_used)]

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const META: &str = env!("CARGO_BIN_EXE_meta");
const IDENTITY: [&str; 6] =
    ["-c", "user.name=test", "-c", "user.email=test@meta.invalid", "-c", "init.defaultBranch=main"];
const CONFIG: &str = "remote = \"origin\"\n";
const EXCLUDED: &str = "target\njunk\n";
const CARGO_TAG: &str =
    "Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag created by cargo.\n";
const ORIGIN: &str = "origin";
const PROJECT: &str = "project";
const DONE: &str = "done";
const NESTED_TARGET: &str = "done/target";
const LOOSE_TARGET: &str = "loose";
const COPY: &str = "copy";
const KINDS: [&str; 3] = ["target-dir", "worktree", "clone"];
const LOGGED: [&str; 3] = ["rev-list", "--count", "meta/log"];
const REMOVALS: &str = "3";
const WORKTREES: [&str; 3] = ["worktree", "list", "--porcelain"];
const COMMIT: [&str; 5] = ["commit", "-q", "--allow-empty", "-m", "work"];
const LINK: &str = "link";
const PLAIN: &str = "plain";
const DIRTY: &str = "dirty";
const UNTRACKED: &str = "untracked";
const IGNORED: &str = "ignored";
const UNPUSHED: &str = "unpushed";
const FORKED: &str = "forked";
const FORK: &str = "fork";
const BUSY: &str = "busy";
const HELD: &str = "held";
const MISSING: &str = "missing";
const REFUSALS: [(&str, &str); 10] = [
    (MISSING, "not found"),
    (LINK, "symlink"),
    (".", "holds the current repo"),
    (PLAIN, "neither a cargo target dir nor a git work tree root"),
    (DIRTY, "uncommitted changes"),
    (UNTRACKED, "untracked files"),
    (IGNORED, "ignored files besides cargo target dirs"),
    (UNPUSHED, "commits not on a remote"),
    (FORKED, "has linked worktrees"),
    (BUSY, "in use"),
];

struct Run(Output);

impl Run {
    fn transcript(&self) -> String {
        [String::from_utf8_lossy(&self.0.stdout), String::from_utf8_lossy(&self.0.stderr)].concat()
    }

    fn succeeded(&self) -> bool {
        self.0.status.success()
    }

    fn refuses(&self, path: &Path, reason: &str) -> bool {
        self.transcript()
            .lines()
            .any(|line| path.to_str().is_some_and(|path| line.contains(path)) && line.ends_with(reason))
    }
}

struct Git;

impl Git {
    fn run(dir: &Path, words: &[&str]) -> Run {
        Run(Command::new("git").arg("-C").arg(dir).args(IDENTITY).args(words).output().unwrap())
    }

    fn worktree(repo: &Path, worktree: PathBuf) -> PathBuf {
        Self::run(repo, &["worktree", "add", "--detach", worktree.to_str().unwrap()]);
        worktree
    }
}

struct CargoTag;

impl CargoTag {
    fn plant(dir: PathBuf) -> PathBuf {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("CACHEDIR.TAG"), CARGO_TAG).unwrap();
        dir
    }
}

struct Scene {
    root: tempfile::TempDir,
}

impl Scene {
    fn new() -> Self {
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

    fn place(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    fn project(&self) -> PathBuf {
        self.place(PROJECT)
    }

    fn meta_rm(&self, paths: &[PathBuf]) -> Run {
        let mut command = Command::new(META);
        command.current_dir(self.project()).arg("rm").args(paths).env("META_ACTOR", "rm-test");
        Run(command.output().unwrap())
    }
}

#[test]
fn a_target_dir_a_pushed_worktree_and_a_pushed_clone_are_removed_and_logged() {
    let scene = Scene::new();
    let worktree = Git::worktree(&scene.project(), scene.place(DONE));
    CargoTag::plant(scene.place(NESTED_TARGET));
    Git::run(scene.root.path(), &["clone", "-q", ORIGIN, COPY]);
    let removed = [CargoTag::plant(scene.place(LOOSE_TARGET)), worktree, scene.place(COPY)];
    let run = scene.meta_rm(&removed);
    let worktrees = Git::run(&scene.project(), &WORKTREES).transcript();
    assert!(removed.iter().all(|path| !path.exists()) && !worktrees.contains(DONE), "{}", run.transcript());
    assert!(KINDS.iter().all(|kind| run.transcript().contains(kind)), "{}", run.transcript());
    assert_eq!(Git::run(&scene.place(ORIGIN), &LOGGED).transcript().trim(), REMOVALS);
}

#[test]
fn each_refused_path_is_kept_with_its_reason() {
    let scene = Scene::new();
    std::os::unix::fs::symlink(scene.project(), scene.place(LINK)).unwrap();
    std::fs::create_dir(scene.place(PLAIN)).unwrap();
    std::fs::write(Git::worktree(&scene.project(), scene.place(DIRTY)).join("meta.toml"), EXCLUDED).unwrap();
    std::fs::write(Git::worktree(&scene.project(), scene.place(UNTRACKED)).join(HELD), CONFIG).unwrap();
    std::fs::write(Git::worktree(&scene.project(), scene.place(IGNORED)).join("junk"), CONFIG).unwrap();
    Git::run(&Git::worktree(&scene.project(), scene.place(UNPUSHED)), &COMMIT);
    Git::run(scene.root.path(), &["clone", "-q", ORIGIN, FORKED]);
    Git::worktree(&scene.place(FORKED), scene.place(FORK));
    let _held = File::create(CargoTag::plant(scene.place(BUSY)).join(HELD)).unwrap();
    let kept: Vec<PathBuf> = REFUSALS.iter().map(|(name, _)| scene.place(name)).collect();
    let run = scene.meta_rm(&kept);
    assert!(REFUSALS.iter().all(|(name, reason)| run.refuses(&scene.place(name), reason)), "{}", run.transcript());
    assert!(
        kept.iter().all(|path| path.exists() || path.ends_with(MISSING)) && !run.succeeded(),
        "{}",
        run.transcript()
    );
}
