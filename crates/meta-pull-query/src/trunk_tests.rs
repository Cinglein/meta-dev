use meta_gh::{GhRun, Outcome};

use crate::{LiveData, LiveReply, Rulebook};

const WORKFLOWS: [&str; 2] = ["ci", "mutants"];
const CONCLUSIONS: [&str; 10] = [
    "SUCCESS",
    "FAILURE",
    "CANCELLED",
    "SKIPPED",
    "TIMED_OUT",
    "ACTION_REQUIRED",
    "NEUTRAL",
    "STALE",
    "STARTUP_FAILURE",
    "NEW_FROM_GITHUB",
];
const SUITE_SLOT: &str = "SUITE";
const SUITE: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}},"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":false},"nodes":[
{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[SUITE]}}]}}}}}}"#;
const MIXED: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}},"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[
{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[
  {"conclusion":null,"workflowRun":{"workflow":{"name":"ci"}}},
  {"conclusion":"CANCELLED","workflowRun":{"workflow":{"name":"mutants"}}},
  {"conclusion":"FAILURE","workflowRun":null}]}},
{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[
  {"conclusion":"TIMED_OUT","workflowRun":{"workflow":{"name":"ci"}}},
  {"conclusion":"SKIPPED","workflowRun":{"workflow":{"name":"mutants"}}}]}},
{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[
  {"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},
  {"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}},
  {"conclusion":"FAILURE","workflowRun":{"workflow":{"name":"docs"}}}]}}]}}}}}}"#;
const SOLO: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}},"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":false},"nodes":[
{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}}]}},
{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"STARTUP_FAILURE","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}}}"#;
const ABSENT: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}}}}"#;
const UNBORN: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}},"trunk":{"defaultBranchRef":null}}}"#;

struct Parsed;

impl Parsed {
    fn trunk(reply: &str) -> String {
        let pulls = serde_json::from_str::<LiveReply>(reply).unwrap().answered().unwrap();
        let data: LiveData = pulls.ruled(Rulebook::default()).unwrap();
        let required = WORKFLOWS.map(|name| name.parse().unwrap()).to_vec();
        data.requiring(required).trunk().iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
    }
}

#[test]
fn the_latest_decided_run_per_required_workflow_wins_and_none_or_no_default_branch_reads_unknown() {
    assert_eq!(Parsed::trunk(MIXED), "ci red, mutants green");
    assert_eq!(Parsed::trunk(SOLO), "ci green, mutants red");
    assert_eq!([ABSENT, UNBORN].map(Parsed::trunk), ["ci unknown, mutants unknown"; 2]);
}

#[test]
fn ci_runs_and_trunk_suites_read_every_github_conclusion_alike() {
    let read = |conclusion: &str| {
        let run = format!(r#"{{"status":"COMPLETED","conclusion":"{conclusion}"}}"#);
        let outcome = serde_json::from_str::<GhRun>(&run).unwrap().outcome().unwrap();
        let suite = format!(r#"{{"conclusion":"{conclusion}","workflowRun":{{"workflow":{{"name":"ci"}}}}}}"#);
        (outcome, Parsed::trunk(&SUITE.replace(SUITE_SLOT, &suite)))
    };
    let alike = |(outcome, trunk): (Outcome, String)| match outcome {
        Outcome::Succeeded => trunk == "ci green, mutants unknown",
        Outcome::Failing => trunk == "ci red, mutants unknown",
        Outcome::Undecided => trunk == "ci unknown, mutants unknown",
    };
    assert_eq!(CONCLUSIONS.map(|conclusion| alike(read(conclusion))), [true; CONCLUSIONS.len()]);
}

#[test]
fn only_the_workflows_the_project_requires_are_read() {
    let pulls = serde_json::from_str::<LiveReply>(MIXED).unwrap().answered().unwrap();
    let data: LiveData = pulls.ruled(Rulebook::default()).unwrap();
    assert_eq!(data.clone().trunk(), []);
    assert_eq!(data.requiring(vec!["docs".parse().unwrap()]).trunk().len(), 1);
}
