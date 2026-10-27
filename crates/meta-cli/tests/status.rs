#![allow(clippy::unwrap_used)]

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

#[path = "support/seeded_log.rs"]
mod seeded_log;
#[path = "support/skeleton.rs"]
mod skeleton;
use seeded_log::{SeededLog, TASK};
use skeleton::Skeleton;

const META: &str = env!("CARGO_BIN_EXE_meta");
const OWNER: &str = "owner";
const STATUS_FLAGS: [&str; 2] = ["status", "--flags"];
const ORPHAN: &str = "orphan #8: open, no task\n";
const TRACED: &str = "on";
const UNTRACED: &str = "";
const TRACE_LAST: [&str; 2] = ["trace", "last"];
const GIT_CALLS: &str = "bin/git.calls";
const TRACES: &str = ".git/meta-local/traces";
const GIT_SPAWN: &str = r#""args":{"program":"git","#;
const COUNTED_LEN: usize = 2;
const FAKE_GH: &str = r#"#!/bin/sh
case "$1 $2" in
"pr list") echo 7 ;;
"api graphql") echo '{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[{"number":7,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},{"number":8,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}]}},"linked":{"pr7":{"number":7,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}},"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}}}' ;;
"api repos/{owner}/{repo}/rules/branches/main") echo '[]' ;;
esac
"#;

struct Project {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
}

impl Project {
    fn new() -> Self {
        let origin = tempfile::tempdir().unwrap();
        let clone = tempfile::tempdir().unwrap();
        Skeleton::Bare.lay(origin.path(), "main");
        Skeleton::Working.lay(clone.path(), TASK);
        let remote = format!(
            "[remote \"origin\"]\n\turl = {}\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n",
            origin.path().display()
        );
        let config = clone.path().join(".git/config");
        std::fs::write(&config, [std::fs::read_to_string(&config).unwrap(), remote].concat()).unwrap();
        std::fs::write(clone.path().join("meta.toml"), "remote = \"origin\"\n").unwrap();
        let bin = clone.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        std::fs::write(bin.join("gh"), FAKE_GH).unwrap();
        let path = std::env::var("PATH").unwrap_or_default();
        let counting =
            format!("#!/bin/sh\n[ -z \"$META_TRACE\" ] || echo >> \"$0.calls\"\nPATH='{path}' exec git \"$@\"\n");
        std::fs::write(bin.join("git"), counting).unwrap();
        let executable = std::fs::metadata("/bin/sh").unwrap().permissions();
        ["gh", "git"]
            .iter()
            .try_for_each(|program| std::fs::set_permissions(bin.join(program), executable.clone()))
            .unwrap();
        Self { origin, clone }
    }

    fn path(&self) -> String {
        format!("{}:{}", self.clone.path().join("bin").display(), std::env::var("PATH").unwrap_or_default())
    }

    fn git_calls_and_spans_len(&self) -> [usize; COUNTED_LEN] {
        let calls = std::fs::read_to_string(self.clone.path().join(GIT_CALLS)).unwrap_or_default().lines().count();
        let trace = std::fs::read_dir(self.clone.path().join(TRACES)).unwrap().next().unwrap().unwrap().path();
        [calls, std::fs::read_to_string(trace).unwrap().matches(GIT_SPAWN).count()]
    }

    fn meta(&self, words: &[&str], stdin: &str, trace: &str) -> Output {
        let mut command = Command::new(META);
        command.current_dir(self.clone.path()).args(words).env("META_ACTOR", OWNER).env("PATH", self.path());
        command.env("META_TRACE", trace);
        let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
        child.wait_with_output().unwrap()
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.clone.path().display())
    }
}

#[test]
fn opening_a_pull_request_from_a_claimed_tasks_branch_links_it_and_status_flags_the_rest() {
    let project = Project::new();
    SeededLog::CLAIMED.plant(project.origin.path());
    let elsewhere = tempfile::tempdir().unwrap();
    let shell = format!("cd {project} && gh pr create --fill");
    let input = serde_json::json!({ "cwd": elsewhere.path(), "tool_input": { "command": shell } }).to_string();
    let linked = String::from_utf8(project.meta(&["hook", "post-tool-use"], &input, UNTRACED).stdout).unwrap();
    let flags = project.meta(&STATUS_FLAGS, "", TRACED);
    assert!(linked.contains(&format!("TaskLinked {TASK} #7")) && !flags.status.success());
    assert_eq!(String::from_utf8(flags.stdout).unwrap(), ORPHAN);
    let [calls, spans] = project.git_calls_and_spans_len();
    let summary = String::from_utf8(project.meta(&TRACE_LAST, "", UNTRACED).stdout).unwrap();
    assert!(calls > 0 && spans == calls && summary.contains(&format!("spawns git {calls}\n")));
}
