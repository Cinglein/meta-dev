#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use crate::fixture::{Fixture, Moment};
use crate::meta_command::MetaCommand;
use crate::recorded_gh::RecordedGh;

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
const SEEDED: Moment = Moment::after_epoch(1_700_000_000);
const MAIN: &str = "refs/heads/main";
const HEADS: &str = "refs/heads/";
const TRACKING: &str = "refs/remotes/origin/";
const LOG: &str = "refs/heads/meta/log";
const EVENTS: &str = r#"{"at":"2026-10-26T00:00:00Z","actor":"owner","kind":"SchemaVersion","version":2}
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
"#;
const LINKED: [&str; 5] = ["1", "2", "3", "4", "5"];
const OPEN_AND_LINKED: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[{"number":2,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}]}},"linked":{"pr1":{"number":1,"state":"MERGED","baseRefName":"main","statusCheckRollup":null},"pr2":{"number":2,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},"pr3":{"number":3,"state":"CLOSED","baseRefName":"main","statusCheckRollup":null},"pr4":{"number":4,"state":"MERGED","baseRefName":"main","statusCheckRollup":null},"pr5":{"number":5,"state":"MERGED","baseRefName":"main","statusCheckRollup":null}}}}
"#;
const RULES: [&str; 4] = [
    "api",
    "repos/{owner}/{repo}/rules/branches/main",
    "--jq",
    "[.[] | select(.type == \"required_status_checks\") | .parameters.required_status_checks[].context]",
];
const CLASSIC: [&str; 4] =
    ["api", "repos/{owner}/{repo}/branches/main", "--jq", "[.protection.required_status_checks.checks[]?.context]"];
const NONE_REQUIRED: &str = "[]\n";
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

const SLEEP: &str = "sleep";
const SECONDS: &str = "30";

struct Line(&'static str, &'static str, &'static str);

struct Sleeper(Child);

impl Drop for Sleeper {
    fn drop(&mut self) {
        drop(self.0.kill());
        drop(self.0.wait());
    }
}

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

struct Settings<'trees>(&'trees Path);

impl fmt::Display for Settings<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "worktree_root = \"{}\"", self.0.display())
    }
}

struct Seeded;

impl Seeded {
    fn origin(dir: &Path) {
        let origin = Fixture::bare(dir, "main");
        let seed = origin.written(MAIN, "meta.toml", CONFIG, &SEEDED);
        origin.branched(HEADS, &ADDED, seed);
        origin.written(LOG, "events.jsonl", EVENTS, &SEEDED);
    }

    fn clone(scene: &Scene) {
        let clone = Fixture::working(scene.clone.path(), scene.origin.path(), "main");
        let seed = clone.written(MAIN, "meta.toml", CONFIG, &SEEDED);
        clone.branched(HEADS, &ADDED, seed);
        clone.branched(TRACKING, &[["main"].as_slice(), &ADDED].concat(), seed);
        clone.worktrees(&scene.trees(), &ADDED, STRAY);
    }
}

struct Scene {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
    trees: tempfile::TempDir,
    gh: RecordedGh,
}

impl Scene {
    fn new() -> Self {
        let scene = Self {
            origin: tempfile::tempdir().unwrap(),
            clone: tempfile::tempdir().unwrap(),
            trees: tempfile::tempdir().unwrap(),
            gh: RecordedGh::new(),
        };
        Seeded::origin(scene.origin.path());
        Seeded::clone(&scene);
        let program = RecordedGh::program();
        std::fs::write(scene.clone.path().join("meta.toml"), format!("{CONFIG}[ci]\nprogram = {program:?}\n")).unwrap();
        std::fs::create_dir_all(scene.clone.path().join(".git/info")).unwrap();
        std::fs::write(scene.clone.path().join(".git/info/exclude"), EXCLUDED).unwrap();
        std::fs::write(scene.clone.path().join("meta.local.toml"), Settings(&scene.trees()).to_string()).unwrap();
        scene.gh.graphql("01-first-linked", &LINKED[..1], OPEN_AND_LINKED).graphql(
            "02-linked",
            &LINKED,
            OPEN_AND_LINKED,
        );
        scene.gh.answered("03-rules", RULES, NONE_REQUIRED).answered("04-classic", CLASSIC, NONE_REQUIRED);
        scene.planted();
        scene
    }

    fn planted(&self) {
        let trees = self.trees();
        std::fs::create_dir_all(trees.join(MERGED).join("target/debug")).unwrap();
        std::fs::write(trees.join(MERGED).join("target/debug/.cargo-lock"), "").unwrap();
        std::fs::write(trees.join(MERGED).join("target/CACHEDIR.TAG"), CARGO_TAG).unwrap();
        let dirty = Fixture::opened(&trees.join(DIRTY));
        dirty.committed("precious");
        dirty.rewound();
        let linked = self.clone.path().join(".git/worktrees");
        PAUSES.iter().try_for_each(|[task, pause]| std::fs::write(linked.join(task).join(pause), DANGLING)).unwrap();
        std::fs::create_dir_all(self.clone.path().join(".git/meta-local")).unwrap();
        std::fs::write(self.clone.path().join(TRAIL), Created(&trees, &[MERGED]).to_string()).unwrap();
    }

    fn trees(&self) -> PathBuf {
        std::fs::canonicalize(self.trees.path()).unwrap()
    }

    fn meta(&self, words: &[&str]) -> String {
        let mut command = MetaCommand::tagged(words);
        command.current_dir(self.clone.path()).env("META_ACTOR", "gc-test");
        command.env("GIT_INDEX_FILE", self.trees.path().join("foreign-index"));
        let output = self.gh.replaying(&mut command).output().unwrap();
        [String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)].concat()
    }
}

#[test]
fn gc_frees_ended_pushed_worktrees_with_their_target_dirs_and_keeps_the_rest_with_reasons() {
    let scene = Scene::new();
    let trees = scene.trees();
    let _held = Sleeper(Command::new(SLEEP).arg(SECONDS).current_dir(trees.join(BUSY)).spawn().unwrap());
    let shows = |shown: &str, lines: &[Line]| lines.iter().all(|line| shown.contains(&Shown(line, &trees).to_string()));
    let freed = Freed(&trees).to_string();
    let ticked = scene.meta(&GC_RUN);
    assert!(ticked.contains(&Shown(&PLANNED, &trees).to_string()) && trees.join(MERGED).exists(), "{ticked}");
    std::fs::write(scene.clone.path().join(TRAIL), Created(&trees, &RECORDED).to_string()).unwrap();
    let swept = scene.meta(&GC);
    let trail = std::fs::read_to_string(scene.clone.path().join(TRAIL)).unwrap();
    let clone = Fixture::opened(scene.clone.path());
    assert!(
        shows(&swept, &SWEPT) && swept.contains(&freed) && !trees.join(MERGED).exists() && !clone.has_branch(MERGED),
        "{swept}"
    );
    assert!(
        KEPT.iter().all(|name| trees.join(name).exists()) && TRAILED.iter().all(|entry| trail.contains(entry)),
        "{trail}"
    );
}
