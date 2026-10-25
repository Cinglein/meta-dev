#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::{Command, Output};

const META: &str = env!("CARGO_BIN_EXE_meta");
const OWNER: &str = "owner";
const ALICE: &str = "alice";
const CONCERN: [&str; 5] = ["concern", "CI", "is", "slow", "--tag"];
const TAG: &str = "acceptance-candidate";
const OPEN_DASHED: [&str; 4] = ["open", "--", "--dashed", "text"];
const ISSUE_WITH_OPTION: [&str; 2] = ["issue", "--verbose"];
const LIST_ALL_CONCERNS: [&str; 3] = ["list", "concerns", "--all"];
const SUMMARY: [&str; 1] = ["list"];
const NOWHERE: [&str; 4] = ["remote", "set-url", "origin", "/nonexistent/origin.git"];
const COUNT: [&str; 3] = ["rev-list", "--count", "meta/log"];
const ISSUE: [&str; 2] = ["issue", "text"];
const OPEN_FOR: [&str; 3] = ["open", "budget", "--addresses"];
const CLAIM: &str = "claim";
const DONE: &str = "done";
const NOTHING_OPEN: &str = "Concerns: 0 open";
const HELD: &str = "held by alice";
const UNKNOWN_OPTION: &str = "unknown option";
const OFFLINE: &str = "offline";
const WRITES: &str = "2";

struct Git;

impl Git {
    const IDENTITY: [&'static str; 6] =
        ["-c", "user.name=test", "-c", "user.email=test@meta.invalid", "-c", "init.defaultBranch=main"];

    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }
}

struct Run(Output);

struct Complaint(String);

struct Written(String);

impl Run {
    const ID_INDEX: usize = 4;

    fn complaint(&self) -> Complaint {
        Complaint(String::from_utf8_lossy(&self.0.stderr).into_owned())
    }

    fn written(&self) -> Written {
        Written(self.to_string().split_whitespace().nth(Self::ID_INDEX).unwrap_or_default().to_owned())
    }
}

impl std::fmt::Display for Run {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&String::from_utf8_lossy(&self.0.stdout))
    }
}

struct Project {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
}

impl Project {
    fn new() -> Self {
        let origin = tempfile::tempdir().unwrap();
        let clone = tempfile::tempdir().unwrap();
        Git::run(origin.path(), &["init", "-q", "--bare"]);
        Git::run(clone.path(), &["init", "-q"]);
        Git::run(clone.path(), &["remote", "add", "origin", origin.path().to_str().unwrap()]);
        std::fs::write(clone.path().join("meta.toml"), "remote = \"origin\"\n").unwrap();
        Self { origin, clone }
    }

    fn meta(&self, actor: &str, words: &[&str]) -> Run {
        Run(Command::new(META).current_dir(self.clone.path()).args(words).env("META_ACTOR", actor).output().unwrap())
    }

    fn task_step(&self, actor: &str, step: &str, task: &Written) -> Run {
        self.meta(actor, &[step, &task.0])
    }
}

#[test]
fn a_task_addresses_a_concern_when_its_claimant_finishes_it() {
    let project = Project::new();
    let concern = project.meta(OWNER, &[&CONCERN[..], &[TAG]].concat()).written();
    let task = project.meta(OWNER, &[&OPEN_FOR[..], &[concern.0.as_str()]].concat()).written();
    project.task_step(ALICE, CLAIM, &task);
    project.task_step(ALICE, DONE, &task);
    assert!(project.meta(OWNER, &LIST_ALL_CONCERNS).to_string().contains(&concern.0));
    assert!(project.meta(OWNER, &SUMMARY).to_string().contains(NOTHING_OPEN));
}

#[test]
fn refusals_and_bad_input_say_why_and_write_nothing() {
    let project = Project::new();
    let task = project.meta(OWNER, &OPEN_DASHED).written();
    project.task_step(ALICE, CLAIM, &task);
    assert!(project.task_step(OWNER, DONE, &task).complaint().0.contains(HELD));
    assert!(project.meta(OWNER, &ISSUE_WITH_OPTION).complaint().0.contains(UNKNOWN_OPTION));
    assert!(Run(Git::run(project.origin.path(), &COUNT)).to_string().starts_with(WRITES));
}

#[test]
fn writes_are_refused_offline() {
    let project = Project::new();
    Git::run(project.clone.path(), &NOWHERE);
    assert!(project.meta(OWNER, &ISSUE).complaint().0.contains(OFFLINE));
}
