#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::Write as _;
use std::process::{Output, Stdio};

#[allow(dead_code)]
#[path = "support/fixture.rs"]
mod fixture;
#[path = "support/meta_command.rs"]
mod meta_command;
#[path = "support/recorded_gh.rs"]
mod recorded_gh;
#[path = "support/seeded_log.rs"]
mod seeded_log;
use fixture::Fixture;
use meta_command::MetaCommand;
use recorded_gh::RecordedGh;
use seeded_log::{SeededLog, TASK};

const OWNER: &str = "owner";
const CLAIMED: &str = "TaskClaimed";
const STATUS_FLAGS: [&str; 2] = ["status", "--flags"];
const ORPHAN: &str = "orphan #8: open, no task\n";
const LINKED: [&str; 1] = ["7"];
const OPEN: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[{"number":7,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},{"number":8,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}]}},"linked":{"pr7":{"number":7,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}},"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}}}
"#;
const HEADED: [&str; 8] = [
    "pr",
    "list",
    "--head",
    TASK,
    "--json",
    "number,isCrossRepository",
    "--jq",
    ".[] | select(.isCrossRepository | not) | .number",
];
const RULES: [&str; 4] = [
    "api",
    "repos/{owner}/{repo}/rules/branches/main",
    "--jq",
    "[.[] | select(.type == \"required_status_checks\") | .parameters.required_status_checks[].context]",
];
const CLASSIC: [&str; 4] =
    ["api", "repos/{owner}/{repo}/branches/main", "--jq", "[.protection.required_status_checks.checks[]?.context]"];
const NONE_REQUIRED: &str = "[]\n";

struct Project {
    origin: tempfile::TempDir,
    clone: tempfile::TempDir,
    gh: RecordedGh,
}

impl Project {
    fn new() -> Self {
        let origin = tempfile::tempdir().unwrap();
        let clone = tempfile::tempdir().unwrap();
        Fixture::bare(origin.path(), "main");
        Fixture::working(clone.path(), origin.path(), TASK);
        let program = RecordedGh::program();
        std::fs::write(clone.path().join("meta.toml"), format!("remote = \"origin\"\n[ci]\nprogram = {program:?}\n"))
            .unwrap();
        let gh = RecordedGh::new();
        gh.answered("01-headed", &HEADED, "7\n")
            .graphql("02-open", &LINKED, OPEN)
            .answered("03-rules", &RULES, NONE_REQUIRED)
            .answered("04-classic", &CLASSIC, NONE_REQUIRED);
        Self { origin, clone, gh }
    }

    fn meta(&self, words: &[&str], stdin: &str) -> Output {
        let mut command = MetaCommand::tagged(words);
        command.current_dir(self.clone.path()).env("META_ACTOR", OWNER);
        let mut child = self.gh.on(&mut command).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
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
    SeededLog::through(CLAIMED).plant(project.origin.path());
    let elsewhere = tempfile::tempdir().unwrap();
    let shell = format!("cd {project} && gh pr create --fill");
    let input = serde_json::json!({ "cwd": elsewhere.path(), "tool_input": { "command": shell } }).to_string();
    let linked = String::from_utf8(project.meta(&["hook", "post-tool-use"], &input).stdout).unwrap();
    let flags = project.meta(&STATUS_FLAGS, "");
    assert!(linked.contains(&format!("TaskLinked {TASK} #7")) && !flags.status.success(), "{linked}");
    assert_eq!(String::from_utf8(flags.stdout).unwrap(), ORPHAN);
}
