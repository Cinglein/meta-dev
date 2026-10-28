#![allow(clippy::unwrap_used)]

use std::fmt;
use std::io::Write as _;

use meta_door::{Change, Door, MetaStore, Opening};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

#[path = "support/skeleton.rs"]
mod skeleton;
use skeleton::Skeleton;

const META: &str = env!("CARGO_BIN_EXE_meta");
const MERGED: &str = "task-00000000000000a1";
const OPEN: &str = "task-00000000000000a2";
const DIRTY: &str = "task-00000000000000a3";
const BUSY: &str = "task-00000000000000a4";
const GONE: &str = "task-00000000000000a5";
const PAUSED: &str = "task-00000000000000a6";
const PICKED: &str = "task-00000000000000a7";
const REVERTED: &str = "task-00000000000000a8";
const REBASED: &str = "task-00000000000000a9";
const PAUSES: [[&str; 2]; 4] =
    [[PAUSED, "MERGE_HEAD"], [PICKED, "CHERRY_PICK_HEAD"], [REVERTED, "REVERT_HEAD"], [REBASED, "REBASE_HEAD"]];
const DANGLING: &str = "1111111111111111111111111111111111111111";
const RECORDED: [&str; 9] = [MERGED, OPEN, DIRTY, BUSY, GONE, PAUSED, PICKED, REVERTED, REBASED];
const ADDED: [&str; 8] = [MERGED, OPEN, DIRTY, BUSY, PAUSED, PICKED, REVERTED, REBASED];
const STRAY: &str = "stray";
const SEED: &str = "commit refs/heads/main
committer t <t@meta.invalid> 1700000000 +0000
data 4
seed
M 100644 inline meta.toml
data <<END
remote = \"origin\"
END

";
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
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a6","text":"x","division":"features-and-maintenance"}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskLinked","task":"task-00000000000000a6","pr":5}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a7","text":"x","division":"features-and-maintenance"}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskLinked","task":"task-00000000000000a7","pr":5}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a8","text":"x","division":"features-and-maintenance"}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskLinked","task":"task-00000000000000a8","pr":5}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000a9","text":"x","division":"features-and-maintenance"}
{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"TaskLinked","task":"task-00000000000000a9","pr":5}
END

"#;
const FAKE_GH: &str = r#"#!/bin/sh
case "$1 $2" in
"api graphql") echo '{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[{"number":2,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}]}},"linked":{"pr1":{"number":1,"state":"MERGED","baseRefName":"main","statusCheckRollup":null},"pr2":{"number":2,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},"pr3":{"number":3,"state":"CLOSED","baseRefName":"main","statusCheckRollup":null},"pr4":{"number":4,"state":"MERGED","baseRefName":"main","statusCheckRollup":null},"pr5":{"number":5,"state":"MERGED","baseRefName":"main","statusCheckRollup":null}}}}' ;;
"api repos/{owner}/{repo}/rules/branches/main") echo '[]' ;;
esac
"#;
const EXCLUDED: &str = "/meta.local.toml\n/bin\ntarget/\n";
const CARGO_TAG: &str =
    "Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag created by cargo.\n";
const GC: [&str; 1] = ["gc"];
const GC_RUN: [&str; 2] = ["heartbeat", "gc-run"];
const PLANNED: Line = Line("would free ", MERGED, ": ");
const CONFIG: &str = "remote = \"origin\"\n";
const IN_PROGRESS: &str = ": am, rebase or cherry-pick in progress; pr #5 merged\n";
const SWEPT: [Line; 10] = [
    Line("kept ", PAUSED, IN_PROGRESS),
    Line("kept ", PICKED, IN_PROGRESS),
    Line("kept ", REVERTED, IN_PROGRESS),
    Line("kept ", REBASED, IN_PROGRESS),
    Line("freed ", MERGED, ": "),
    Line("kept ", OPEN, ": pr #2 open\n"),
    Line("kept ", DIRTY, ": commits not on the remote; pr #3 closed\n"),
    Line("kept ", BUSY, ": in use; pr #4 merged\n"),
    Line("unrecorded ", STRAY, ": left alone\n"),
    Line("closed ", GONE, ": missing\n"),
];
const TRAIL: &str = "meta-local/worktrees.jsonl";
const TRAIL_DIR: &str = "meta-local";
const TRAILED: [&str; 3] = ["\"change\":\"Freed\"", "\"change\":\"Refused\"", "\"change\":\"Closed\""];
const KEPT: [&str; 8] = [OPEN, DIRTY, BUSY, STRAY, PAUSED, PICKED, REVERTED, REBASED];

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
        let linked = Door::read(worktree.join(".git")).unwrap_or_default();
        let bytes = [linked.as_str(), CONFIG, CARGO_TAG].concat().len();
        writeln!(formatter, "freed {}: {bytes} bytes; pr #1 merged", worktree.display())
    }
}

struct Created<'trees>(&'trees Path, &'static [&'static str]);

impl fmt::Display for Created<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let created = "2026-10-26T00:00:00Z";
        self.1.iter().try_for_each(|task| {
            writeln!(
                formatter,
                r#"{{"change":"Created","path":"{}","branch":"{task}","task":"{task}","created":"{created}","worktree":"{task}"}}"#,
                self.0.join(task).display()
            )
        })
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

    fn imported(repo: &Path, stream: &str) {
        let mut command = Command::new("git");
        let piped = command.arg("-C").arg(repo).args(["fast-import", "--quiet"]).stdin(Stdio::piped());
        let mut child = piped.spawn().unwrap();
        child.stdin.take().unwrap().write_all(stream.as_bytes()).unwrap();
        assert!(child.wait().unwrap().success());
    }

    fn seeded(origin: &Path, clone: &Path) {
        let branches = |prefix: &str, names: &[&str]| -> String {
            names.iter().flat_map(|name| ["reset ", prefix, name, "\nfrom refs/heads/main\n\n"]).collect()
        };
        let heads = branches("refs/heads/", &ADDED);
        let tracking = branches("refs/remotes/origin/", &[["main"].as_slice(), &ADDED].concat());
        std::thread::scope(|scope| {
            scope.spawn(|| Self::imported(origin, &[SEED, &heads, LOG].concat()));
            Self::imported(clone, &[SEED, &heads, &tracking].concat());
        });
    }

    fn added(scene: &Scene) {
        let clone = scene.clone.path();
        let trees = scene.trees();
        let added: Vec<Output> = ADDED
            .iter()
            .map(|task| Self::run(clone, &["worktree", "add", "-q", trees.join(task).to_str().unwrap(), task]))
            .collect();
        assert!(added.iter().all(|output| output.status.success()));
        Self::run(clone, &["worktree", "add", "-q", "--detach", trees.join(STRAY).to_str().unwrap()]);
    }
}

struct Scene {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
    trees: tempfile::TempDir,
    excludes: tempfile::NamedTempFile,
}

impl Scene {
    fn new() -> Self {
        let scene = Self {
            origin: tempfile::tempdir().unwrap(),
            clone: tempfile::tempdir().unwrap(),
            trees: tempfile::tempdir().unwrap(),
            excludes: tempfile::NamedTempFile::new().unwrap(),
        };
        Skeleton::laid(scene.origin.path(), scene.clone.path(), "main");
        let clone = scene.clone.path();
        Door::write(scene.excludes.path(), EXCLUDED).unwrap();
        let excludes = scene.excludes.path().display().to_string();
        assert!(Git::run(clone, &["config", "core.excludesFile", &excludes]).status.success());
        Door::write(clone.join("meta.toml"), CONFIG).unwrap();
        Door::write(clone.join("meta.local.toml"), Settings(&scene.trees()).to_string()).unwrap();
        Door::changed(clone.join("bin"), Change::Dir).unwrap();
        Door::write(clone.join("bin/gh"), FAKE_GH).unwrap();
        let executable = Path::new("/bin/sh").metadata().unwrap().permissions();
        Door::changed(clone.join("bin/gh"), Change::Permissions(executable)).unwrap();
        Git::seeded(scene.origin.path(), scene.clone.path());
        Git::added(&scene);
        scene.planted();
        scene
    }

    fn planted(&self) {
        let trees = self.trees();
        Door::changed(trees.join(MERGED).join("target/debug"), Change::DirAll).unwrap();
        Door::write(trees.join(MERGED).join("target/debug/.cargo-lock"), "").unwrap();
        Door::write(trees.join(MERGED).join("target/CACHEDIR.TAG"), CARGO_TAG).unwrap();
        Git::run(&trees.join(DIRTY), &["commit", "-q", "--allow-empty", "-m", "precious"]);
        Git::run(&trees.join(DIRTY), &["update-ref", "WIP", "HEAD"]);
        Git::run(&trees.join(DIRTY), &["reset", "-q", "--hard", "HEAD~1"]);
        Git::run(&trees.join(DIRTY), &["update-ref", "-d", "ORIG_HEAD"]);
        PAUSES.iter().try_for_each(|[task, pause]| Door::dangling_pause(&trees.join(task), pause, DANGLING)).unwrap();
        Door::changed(self.store().entry(TRAIL_DIR), Change::DirAll).unwrap();
        Door::write(self.store().entry(TRAIL), Created(&trees, &[MERGED]).to_string()).unwrap();
    }

    fn trees(&self) -> PathBuf {
        self.trees.path().canonicalize().unwrap()
    }

    pub fn store(&self) -> MetaStore {
        MetaStore::located(self.clone.path()).unwrap()
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
    let _held = Door::opened(trees.join(BUSY).join(".git"), Opening::Read).unwrap();
    let shows = |shown: &str, lines: &[Line]| lines.iter().all(|line| shown.contains(&Shown(line, &trees).to_string()));
    let freed = Freed(&trees).to_string();
    let ticked = scene.meta(&GC_RUN);
    assert!(ticked.contains(&Shown(&PLANNED, &trees).to_string()) && trees.join(MERGED).exists(), "{ticked}");
    Door::write(scene.store().entry(TRAIL), Created(&trees, &RECORDED).to_string()).unwrap();
    let swept = scene.meta(&GC);
    let trail = Door::read(scene.store().entry(TRAIL)).unwrap();
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
