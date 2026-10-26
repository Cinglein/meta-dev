#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::{Command, Output};

const META: &str = env!("CARGO_BIN_EXE_meta");
const OWNER: &str = "owner";
const NOWHERE: [&str; 4] = ["remote", "set-url", "origin", "/nonexistent/origin.git"];
const COUNT: [&str; 3] = ["rev-list", "--count", "meta/log"];
const ISSUE: [&str; 2] = ["issue", "text"];
const OFFLINE: &str = "offline";
const WRITES: &str = "1";
const LIST_ISSUES: [&str; 2] = ["list", "issues"];

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
}

#[test]
fn a_write_lands_on_the_remote_and_offline_writes_are_refused() {
    let project = Project::new();
    let issue = project.meta(OWNER, &ISSUE).written();
    assert!(project.meta(OWNER, &LIST_ISSUES).to_string().contains(&issue.0));
    assert!(Run(Git::run(project.origin.path(), &COUNT)).to_string().starts_with(WRITES));
    Git::run(project.clone.path(), &NOWHERE);
    assert!(project.meta(OWNER, &ISSUE).complaint().0.contains(OFFLINE));
}
