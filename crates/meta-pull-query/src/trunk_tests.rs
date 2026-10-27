use crate::{LiveData, LiveReply, Rulebook};

const WORKFLOWS: [&str; 2] = ["ci", "mutants"];
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
fn the_latest_decided_run_of_each_required_workflow_wins_and_undecided_and_unrelated_runs_are_skipped() {
    assert_eq!(Parsed::trunk(MIXED), "ci red, mutants green");
    assert_eq!(Parsed::trunk(SOLO), "ci green, mutants red");
}

#[test]
fn a_workflow_with_no_decided_run_in_the_window_or_no_default_branch_reads_unknown() {
    assert_eq!(Parsed::trunk(ABSENT), "ci unknown, mutants unknown");
    assert_eq!(Parsed::trunk(UNBORN), "ci unknown, mutants unknown");
}

#[test]
fn only_the_workflows_the_project_requires_are_read() {
    let pulls = serde_json::from_str::<LiveReply>(MIXED).unwrap().answered().unwrap();
    let data: LiveData = pulls.ruled(Rulebook::default()).unwrap();
    assert_eq!(data.clone().trunk(), []);
    assert_eq!(data.requiring(vec!["docs".parse().unwrap()]).trunk().len(), 1);
}
