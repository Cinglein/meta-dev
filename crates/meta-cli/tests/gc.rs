#![allow(clippy::unwrap_used)]

use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/world.rs"]
mod world;
use world::World;

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
const DANGLING: &str = "1111111111111111111111111111111111111111\n";
const RECORDED: [&str; 9] = [MERGED, OPEN, DIRTY, BUSY, GONE, PAUSED, PICKED, REVERTED, REBASED];
const ADDED: [&str; 8] = [MERGED, OPEN, DIRTY, BUSY, PAUSED, PICKED, REVERTED, REBASED];
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
"api repos/{owner}/{repo}/rules/branches/main"|"api repos/{owner}/{repo}/branches/main") echo '[]' ;;
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
const TRAIL: &str = ".git/meta-local/worktrees.jsonl";
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
        let linked = std::fs::read_to_string(worktree.join(".git")).unwrap_or_default();
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

struct Scene {
    _world: World,
    clone: PathBuf,
    trees: PathBuf,
}

impl Scene {
    fn new() -> Self {
        let world = World::new(&ADDED, LOG);
        world.added(&ADDED);
        let stray = world.trees().join(STRAY);
        World::run(world.clone.path(), &["worktree", "add", "-q", "--detach", stray.to_str().unwrap()]);
        let scene =
            Self { clone: world.clone.path().to_path_buf(), trees: world.trees.path().to_path_buf(), _world: world };
        std::fs::create_dir(scene.clone.join(".git/info")).unwrap();
        std::fs::write(scene.clone.join(".git/info/exclude"), EXCLUDED).unwrap();
        std::fs::write(scene.clone.join("meta.local.toml"), Settings(&scene.trees()).to_string()).unwrap();
        std::fs::create_dir(scene.clone.join("bin")).unwrap();
        std::fs::write(scene.clone.join("bin/gh"), FAKE_GH).unwrap();
        std::fs::set_permissions(
            scene.clone.join("bin/gh"),
            std::fs::metadata("/bin/sh").unwrap().permissions(),
        )
        .unwrap();
        scene.planted();
        scene
    }

    fn planted(&self) {
        let trees = self.trees();
        std::fs::create_dir_all(trees.join(MERGED).join("target/debug")).unwrap();
        std::fs::write(trees.join(MERGED).join("target/debug/.cargo-lock"), "").unwrap();
        std::fs::write(trees.join(MERGED).join("target/CACHEDIR.TAG"), CARGO_TAG).unwrap();
        World::run(&trees.join(DIRTY), &["commit", "-q", "--allow-empty", "-m", "precious"]);
        World::run(&trees.join(DIRTY), &["update-ref", "WIP", "HEAD"]);
        World::run(&trees.join(DIRTY), &["reset", "-q", "--hard", "HEAD~1"]);
        World::run(&trees.join(DIRTY), &["update-ref", "-d", "ORIG_HEAD"]);
        let linked = self.clone.join(".git/worktrees");
        PAUSES.iter().try_for_each(|[task, pause]| std::fs::write(linked.join(task).join(pause), DANGLING)).unwrap();
        std::fs::create_dir_all(self.clone.join(".git/meta-local")).unwrap();
        std::fs::write(self.clone.join(TRAIL), Created(&trees, &[MERGED]).to_string()).unwrap();
    }

    fn trees(&self) -> PathBuf {
        std::fs::canonicalize(&self.trees).unwrap()
    }

    fn meta(&self, words: &[&str]) -> String {
        let mut command = Command::new(META);
        let searched = Searched(&self.clone.join("bin")).to_string();
        command.current_dir(&self.clone).args(words).env("META_ACTOR", "gc-test").env("PATH", searched);
        command.env("GIT_INDEX_FILE", self.trees.join("foreign-index"));
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
    let ticked = scene.meta(&GC_RUN);
    assert!(ticked.contains(&Shown(&PLANNED, &trees).to_string()) && trees.join(MERGED).exists(), "{ticked}");
    std::fs::write(scene.clone.join(TRAIL), Created(&trees, &RECORDED).to_string()).unwrap();
    let swept = scene.meta(&GC);
    let trail = std::fs::read_to_string(scene.clone.join(TRAIL)).unwrap();
    let branch = World::run(&scene.clone, &["branch", "--list", MERGED]).stdout;
    assert!(
        shows(&swept, &SWEPT) && swept.contains(&freed) && !trees.join(MERGED).exists() && branch.is_empty(),
        "{swept}"
    );
    assert!(
        KEPT.iter().all(|name| trees.join(name).exists()) && TRAILED.iter().all(|entry| trail.contains(entry)),
        "{trail}"
    );
}
