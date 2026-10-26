#![allow(clippy::unwrap_used)]

use std::fmt;
use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const META: &str = env!("CARGO_BIN_EXE_meta");
const MERGED: &str = "task-00000000000000a1";
const OPEN: &str = "task-00000000000000a2";
const DIRTY: &str = "task-00000000000000a3";
const BUSY: &str = "task-00000000000000a4";
const GONE: &str = "task-00000000000000a5";
const RECORDED: [&str; 5] = [MERGED, OPEN, DIRTY, BUSY, GONE];
const ADDED: [&str; 4] = [MERGED, OPEN, DIRTY, BUSY];
const STRAY: &str = "stray";
const LOG: &str = r#"commit refs/heads/meta/log
committer t <t@meta.invalid> 1700000000 +0000
data 4
seed
M 100644 inline events.jsonl
data <<END
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"SchemaVersion","version":2}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a1","text":"x","division":"features-and-maintenance"}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a2","text":"x","division":"features-and-maintenance"}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a3","text":"x","division":"features-and-maintenance"}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a4","text":"x","division":"features-and-maintenance"}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a5","text":"x","division":"features-and-maintenance"}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskLinked","task":"task-00000000000000a1","pr":1}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskLinked","task":"task-00000000000000a2","pr":2}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskLinked","task":"task-00000000000000a3","pr":3}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskLinked","task":"task-00000000000000a4","pr":4}
END

"#;
const FAKE_GH: &str = r#"#!/bin/sh
case "$1 $2" in
"api graphql") echo '{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[{"number":2,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}]}},"linked":{"pr1":{"number":1,"state":"MERGED","baseRefName":"main","statusCheckRollup":null},"pr2":{"number":2,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},"pr3":{"number":3,"state":"CLOSED","baseRefName":"main","statusCheckRollup":null},"pr4":{"number":4,"state":"MERGED","baseRefName":"main","statusCheckRollup":null}}}}' ;;
"api repos/{owner}/{repo}/rules/branches/main") echo '[]' ;;
esac
"#;
const EXCLUDED: &str = "/meta.local.toml\n/bin\ntarget/\n";
const CARGO_TAG: &str =
    "Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag created by cargo.\n";
const GC: [&str; 1] = ["gc"];
const CONFIG: &str = "remote = \"origin\"\n";
const SWEPT: [Line; 6] = [
    Line("freed ", MERGED, ": "),
    Line("kept ", OPEN, ": pr #2 open\n"),
    Line("kept ", DIRTY, ": symlink; pr #3 closed\n"),
    Line("kept ", BUSY, ": in use; pr #4 merged\n"),
    Line("unrecorded ", STRAY, ": left alone\n"),
    Line("closed ", GONE, ": missing\n"),
];
const TRAIL: &str = ".git/meta-local/worktrees.jsonl";
const TRAILED: [&str; 3] = ["\"change\":\"Freed\"", "\"change\":\"Refused\"", "\"change\":\"Closed\""];
const KEPT: [&str; 4] = [OPEN, DIRTY, BUSY, STRAY];

struct Line(&'static str, &'static str, &'static str);

struct Shown<'line>(&'line Line, &'line Path);

impl fmt::Display for Shown<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Line(verb, name, tail) = self.0;
        write!(formatter, "{verb}{}{tail}", self.1.join(name).display())
    }
}

struct Freed<'trees>(&'trees Path);

impl fmt::Display for Freed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let worktree = self.0.join(MERGED);
        let linked = std::fs::read_to_string(worktree.join(".git")).unwrap_or_default();
        let bytes = [linked.as_str(), CONFIG, CARGO_TAG].concat().len();
        writeln!(formatter, "freed {}: {bytes} bytes; pr #1 merged", worktree.display())
    }
}

struct Created<'trees>(&'trees Path, &'static str);

impl fmt::Display for Created<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let task = self.1;
        let created = "2026-10-26T00:00:00Z";
        writeln!(
            formatter,
            r#"{{"change":"Created","path":"{}","branch":"{task}","task":"{task}","created":"{created}"}}"#,
            self.0.join(task).display()
        )
    }
}

struct Searched<'bin>(&'bin Path);

impl fmt::Display for Searched<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.0.display(), std::env::var("PATH").unwrap_or_default())
    }
}

struct Settings<'trees>(&'trees Path);

impl fmt::Display for Settings<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "worktree_root = \"{}\"", self.0.display())
    }
}

struct Git;

impl Git {
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

    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }

    fn logged(bare: &Path) {
        let mut command = Command::new("git");
        let piped = command.arg("-C").arg(bare).args(["fast-import", "--quiet"]).stdin(Stdio::piped());
        let mut child = piped.spawn().unwrap();
        child.stdin.take().unwrap().write_all(LOG.as_bytes()).unwrap();
        assert!(child.wait().unwrap().success());
    }

    fn added(clone: &Path, trees: &Path) {
        let added: Vec<Output> = ADDED
            .iter()
            .map(|task| Self::run(clone, &["worktree", "add", "-q", "-b", task, trees.join(task).to_str().unwrap()]))
            .collect();
        assert!(added.iter().all(|output| output.status.success()));
        let pushed = ["push", "-q", "origin", "HEAD:refs/heads/main"].into_iter().chain(ADDED);
        Self::run(clone, &pushed.collect::<Vec<_>>());
        Self::run(clone, &["worktree", "add", "-q", "--detach", trees.join(STRAY).to_str().unwrap()]);
    }
}

struct Scene {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
    trees: tempfile::TempDir,
}

impl Scene {
    fn new() -> Self {
        let scene = Self {
            origin: tempfile::tempdir().unwrap(),
            clone: tempfile::tempdir().unwrap(),
            trees: tempfile::tempdir().unwrap(),
        };
        Git::run(scene.origin.path(), &["init", "-q", "--bare"]);
        Git::run(scene.clone.path(), &["init", "-q"]);
        Git::run(scene.clone.path(), &["remote", "add", "origin", scene.origin.path().to_str().unwrap()]);
        std::fs::write(scene.clone.path().join("meta.toml"), CONFIG).unwrap();
        Git::run(scene.clone.path(), &["add", "meta.toml"]);
        Git::run(scene.clone.path(), &["commit", "-q", "-m", "seed"]);
        std::fs::write(scene.clone.path().join(".git/info/exclude"), EXCLUDED).unwrap();
        std::fs::write(scene.clone.path().join("meta.local.toml"), Settings(&scene.trees()).to_string()).unwrap();
        std::fs::create_dir(scene.clone.path().join("bin")).unwrap();
        std::fs::write(scene.clone.path().join("bin/gh"), FAKE_GH).unwrap();
        std::fs::set_permissions(
            scene.clone.path().join("bin/gh"),
            std::fs::metadata("/bin/sh").unwrap().permissions(),
        )
        .unwrap();
        Git::logged(scene.origin.path());
        Git::added(scene.clone.path(), &scene.trees());
        scene.planted();
        scene
    }

    fn planted(&self) {
        let trees = self.trees();
        std::fs::create_dir_all(trees.join(MERGED).join("target/debug")).unwrap();
        std::fs::write(trees.join(MERGED).join("target/debug/.cargo-lock"), "").unwrap();
        std::fs::write(trees.join(MERGED).join("target/CACHEDIR.TAG"), CARGO_TAG).unwrap();
        std::fs::create_dir(trees.join(DIRTY).join("target")).unwrap();
        std::os::unix::fs::symlink(&trees, trees.join(DIRTY).join("target/trees")).unwrap();
        std::fs::create_dir_all(self.clone.path().join(".git/meta-local")).unwrap();
        let trail: String = RECORDED.iter().map(|task| Created(&trees, task).to_string()).collect();
        std::fs::write(self.clone.path().join(TRAIL), trail).unwrap();
    }

    fn trees(&self) -> PathBuf {
        std::fs::canonicalize(self.trees.path()).unwrap()
    }

    fn meta(&self, words: &[&str]) -> String {
        let mut command = Command::new(META);
        let searched = Searched(&self.clone.path().join("bin")).to_string();
        command.current_dir(self.clone.path()).args(words).env("META_ACTOR", "gc-test").env("PATH", searched);
        command.env("GIT_INDEX_FILE", self.trees.path().join("foreign-index"));
        let output = command.output().unwrap();
        [String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)].concat()
    }
}

#[test]
fn gc_frees_ended_pushed_worktrees_with_their_target_dirs_and_keeps_the_rest_with_reasons() {
    let scene = Scene::new();
    let trees = scene.trees();
    let _held = File::open(trees.join(BUSY).join(".git")).unwrap();
    let shows = |shown: &str, lines: &[Line]| lines.iter().all(|line| shown.contains(&Shown(line, &trees).to_string()));
    let freed = Freed(&trees).to_string();
    let swept = scene.meta(&GC);
    let trail = std::fs::read_to_string(scene.clone.path().join(TRAIL)).unwrap();
    let branch = Git::run(scene.clone.path(), &["branch", "--list", MERGED]).stdout;
    assert!(
        shows(&swept, &SWEPT) && swept.contains(&freed) && !trees.join(MERGED).exists() && branch.is_empty(),
        "{swept}"
    );
    assert!(
        KEPT.iter().all(|name| trees.join(name).exists()) && TRAILED.iter().all(|entry| trail.contains(entry)),
        "{trail}"
    );
}
