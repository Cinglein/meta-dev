#![allow(clippy::unwrap_used)]

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};

const META: &str = env!("CARGO_BIN_EXE_meta");
const OWNER: &str = "owner";
const WRITER: &str = "writer";
const NOWHERE: [&str; 4] = ["remote", "set-url", "origin", "/nonexistent/origin.git"];
const COUNT: [&str; 3] = ["rev-list", "--count", "meta/log"];
const ISSUE: [&str; 4] = ["issue", "--division", "features-and-maintenance", "text"];
const OFFLINE: &str = "offline";
const WRITES: &str = "2";
const LIST_ISSUES: [&str; 2] = ["list", "issues"];
const LOCAL_NAME: &str = "localname";
const OPEN: [&str; 4] = ["open", "--division", "features-and-maintenance", "text"];
const PR_CREATE: &str = "gh pr create --fill";
const STATUS_FLAGS: [&str; 2] = ["status", "--flags"];
const ORPHAN: &str = "orphan #8: open, no task\n";
const FAKE_GH: &str = r#"#!/bin/sh
case "$1" in
pr) echo 7 ;;
api) echo '{"data":{"open":{"pullRequests":{"nodes":[{"number":7},{"number":8}]}},"linked":{"pr7":{"state":"OPEN","statusCheckRollup":null}}}}' ;;
esac
"#;

struct Git;

impl Git {
    const IDENTITY: [&'static str; 6] =
        ["-c", "user.name=test", "-c", "user.email=test@meta.invalid", "-c", "init.defaultBranch=main"];

    fn run(dir: &Path, words: &[&str]) -> Output {
        Command::new("git").arg("-C").arg(dir).args(Self::IDENTITY).args(words).output().unwrap()
    }

    fn link_origin(dir: &Path, url: &str) {
        let remote = format!("[remote \"origin\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n");
        std::fs::OpenOptions::new()
            .append(true)
            .open(dir.join(".git/config"))
            .unwrap()
            .write_all(remote.as_bytes())
            .unwrap();
    }
}

struct Run(Output);

struct Complaint(String);

struct Written(String);

impl Run {
    const ACTOR_INDEX: usize = 2;
    const ID_INDEX: usize = 4;

    fn actor(&self) -> Option<String> {
        self.to_string().split_whitespace().nth(Self::ACTOR_INDEX).map(str::to_owned)
    }

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
        Git::link_origin(clone.path(), origin.path().to_str().unwrap());
        std::fs::write(clone.path().join("meta.toml"), "remote = \"origin\"\n").unwrap();
        let bin = clone.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        std::fs::write(bin.join("gh"), FAKE_GH).unwrap();
        std::fs::set_permissions(bin.join("gh"), std::fs::metadata("/bin/sh").unwrap().permissions()).unwrap();
        Self { origin, clone }
    }

    fn meta(&self, actor: &str, words: &[&str]) -> Run {
        let path = format!("{}:{}", self.clone.path().join("bin").display(), std::env::var("PATH").unwrap_or_default());
        let mut command = Command::new(META);
        command.current_dir(self.clone.path()).args(words).env("META_ACTOR", actor).env("PATH", path);
        Run(command.output().unwrap())
    }

    fn post_tool_use(&self, shell: &str) -> Run {
        let input = serde_json::json!({ "cwd": self.clone.path(), "tool_input": { "command": shell } }).to_string();
        let mut child = Command::new(META)
            .args(["hook", "post-tool-use"])
            .env("META_ACTOR", OWNER)
            .env("PATH", format!("{}:{}", self.clone.path().join("bin").display(), std::env::var("PATH").unwrap()))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        Run(child.wait_with_output().unwrap())
    }

    fn meta_unset(&self, words: &[&str]) -> Run {
        let mut command = Command::new(META);
        command.current_dir(self.clone.path()).args(words).env_remove("META_ACTOR").env("USER", LOCAL_NAME);
        Run(command.output().unwrap())
    }
}

#[test]
fn writes_go_to_the_remote_as_meta_actor_else_owner_and_offline_writes_are_refused() {
    let project = Project::new();
    let opened = project.meta_unset(&ISSUE);
    let listed = project.meta(OWNER, &LIST_ISSUES).to_string();
    assert!(opened.actor().as_deref() == Some(OWNER) && listed.contains(&opened.written().0));
    assert!(Run(Git::run(project.origin.path(), &COUNT)).to_string().starts_with(WRITES));
    assert_eq!(project.meta(WRITER, &ISSUE).actor().as_deref(), Some(WRITER));
    Git::run(project.clone.path(), &NOWHERE);
    assert!(project.meta(OWNER, &ISSUE).complaint().0.contains(OFFLINE));
}

#[test]
fn opening_a_pull_request_from_a_claimed_tasks_branch_links_it_and_status_flags_the_rest() {
    let project = Project::new();
    let task = project.meta(OWNER, &OPEN).written().0;
    project.meta(OWNER, &["claim", &task]);
    Git::run(project.clone.path(), &["switch", "-q", "-c", &format!("status/{task}")]);
    let linked = project.post_tool_use(PR_CREATE).to_string();
    let flags = project.meta(OWNER, &STATUS_FLAGS);
    assert!(linked.contains(&format!("TaskLinked {task} #7")) && !flags.0.status.success());
    assert_eq!(flags.to_string(), ORPHAN);
}
