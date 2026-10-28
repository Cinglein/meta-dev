use meta_pull_query::LiveReply;
use meta_schema::{Actor, Block, Blocker, Claim, Event, Id, NewIssue, NewTask, Record, Task, TaskLink, TaskRef};
use meta_state::State;

use super::Status;
use crate::github::Github;
use crate::github_failure::GithubFailure;

const ALICE: &str = "alice";
pub(super) const NOW: &str = "2026-10-26T12:00:00Z";
pub(super) const PAST: &str = "2026-10-01T00:00:00Z";
const EFFORT: &str = "5000";
const CI: &str = "features-and-maintenance/ci";
const FEATURES: &str = "features-and-maintenance";
const PERFORMANCE: &str = "performance-and-cost";
const LONG_TEXT: &str = "one two three four five six seven eight nine";
const LEGACY: &str = r#"{"task":"task-00000000000000c0","text":"legacy root task"}"#;
const REPLY: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":true},"nodes":[
{"number":13,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},
{"number":14,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},
{"number":15,"state":"OPEN","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[{"name":"ci","result":"FAILURE"}]}}},
{"number":16,"state":"OPEN","baseRefName":"main","statusCheckRollup":null}]}},"linked":{
"pr13":{"number":13,"state":"OPEN","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[{"name":"ci","result":"FAILURE"}]}}},
"pr14":{"number":14,"state":"OPEN","baseRefName":"main","statusCheckRollup":{"contexts":{"pageInfo":{"hasNextPage":true},"nodes":[{"name":"ci","result":"FAILURE"}]}}},
"pr99":null},
"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}}}"#;
const WORKFLOWS: [&str; 2] = ["ci", "mutants"];
const MAIN: &str = r#""main""#;
const REQUIRED: &str = r#"["ci"]"#;
const TIMED_OUT: &str = "timed out";
const NOTHING_OPEN: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}},"trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}}}"#;

#[derive(Default)]
struct Board {
    records: Vec<Record>,
}

impl Board {
    fn then(mut self, event: Event) -> Self {
        self.records.push(Record::new(ALICE.parse::<Actor>().unwrap(), event));
        self
    }

    fn linked(self, task: Id<Task>, number: &str) -> Self {
        self.then(Event::TaskLinked(TaskLink::new(task, number.parse().unwrap())))
    }

    fn claimed(self, task: Id<Task>) -> Self {
        self.then(Event::TaskClaimed(Claim::new(task, None)))
    }

    fn task(division: &str, text: &str) -> NewTask {
        NewTask::new(division.parse().unwrap(), text.parse().unwrap(), vec![]).due_by(PAST.parse().unwrap())
    }
}

pub(super) struct Tasks {
    held: NewTask,
    late: NewTask,
    root: NewTask,
    discussed: NewTask,
    finished: NewTask,
    issue: NewIssue,
}

impl Tasks {
    fn new() -> Self {
        Self {
            issue: NewIssue::new(FEATURES.parse().unwrap(), "talk".parse().unwrap()),
            held: Board::task(CI, "held"),
            late: Board::task(FEATURES, LONG_TEXT).allowing(EFFORT.parse().unwrap()),
            root: serde_json::from_str(LEGACY).unwrap(),
            discussed: Board::task(PERFORMANCE, "discussed"),
            finished: Board::task(FEATURES, "finished"),
        }
    }

    fn state(&self) -> State {
        let issue = &self.issue;
        let board = [&self.held, &self.late, &self.root, &self.discussed, &self.finished]
            .into_iter()
            .fold(Board::default().then(Event::IssueReported(issue.clone())), |board, task| {
                board.then(Event::TaskOpened(task.clone()))
            });
        let discussed = self.discussed.task();
        let board = board.claimed(self.held.task()).linked(self.held.task(), "13").linked(self.late.task(), "14");
        let board = board.linked(self.root.task(), "99").claimed(discussed);
        let board = board.then(Event::TaskBlocked(Block::new(discussed, Blocker::Discussion(issue.issue()))));
        let board = board.claimed(self.finished.task());
        let board = board
            .linked(self.finished.task(), "15")
            .linked(self.finished.task(), "17")
            .then(Event::TaskDone(TaskRef::new(self.finished.task())));
        State::fold(&board.records)
    }

    pub(super) fn reached(reply: &str, required: &str) -> Github {
        let rules = [(serde_json::from_str(MAIN).unwrap(), serde_json::from_str(required).unwrap())];
        let pulls = serde_json::from_str::<LiveReply>(reply).unwrap().answered().unwrap();
        let required = WORKFLOWS.map(|name| name.parse().unwrap()).to_vec();
        Github::Reached(pulls.ruled(rules.into_iter().collect()).unwrap().requiring(required))
    }

    fn done(&self, state: &State) -> String {
        let finished = state.tasks().iter().find(|task| task.new_task().task() == self.finished.task());
        finished.map(|task| state.status_of(task).to_string()).unwrap_or_default()
    }
}

#[test]
fn flags_name_late_red_unknown_truncated_and_orphan_pull_requests_and_red_ones_on_done_tasks() {
    let tasks = Tasks::new();
    let state = tasks.state();
    let github = Tasks::reached(REPLY, REQUIRED);
    let flags = Status::new(&state, &github, NOW.parse().unwrap()).flags().to_string();
    let held = tasks.held.task();
    let late = tasks.late.task();
    let finished = tasks.finished.task();
    let expected = format!(
        "truncated: more than 100 open pull requests\npast due {held}: due 2026-10-01 00:00Z\nstalled {late}: past due 2026-10-01 00:00Z, no holder\ntruncated #14 {late}: more than 100 checks\nunknown #99 {}: not on github\nred #15 {finished}: no holder\norphan #16: open, no task\n",
        tasks.root.task()
    );
    assert_eq!(flags, expected);
}

#[test]
fn the_tree_runs_from_divisions_down_to_tasks_with_pull_requests_holders_effort_and_due() {
    let tasks = Tasks::new();
    let state = tasks.state();
    let github = Tasks::reached(REPLY, REQUIRED);
    let shown = Status::new(&state, &github, NOW.parse().unwrap()).to_string();
    let tree = shown.split_once("orphan #16: open, no task\n").map(|(_, tree)| tree).unwrap_or_default();
    let expected = format!(
        "root\n  {} open | #99 unknown | no holder | legacy root task\nfeatures-and-maintenance\n  {} open | #14 open checks truncated | no holder | effort 5000 tokens, spend unmeasured | due 2026-10-01 00:00Z | one two three four five six seven eight…\n  {} {} | #15 open required red | no holder | due 2026-10-01 00:00Z | finished\n  features-and-maintenance/ci\n    {} assigned | #13 open required red | held by alice | due 2026-10-01 00:00Z | held\nperformance-and-cost\n  {} in discussion {} | held by alice | due 2026-10-01 00:00Z | discussed\n",
        tasks.root.task(),
        tasks.late.task(),
        tasks.finished.task(),
        tasks.done(&state),
        tasks.held.task(),
        tasks.discussed.task(),
        tasks.issue.issue()
    );
    assert_eq!(tree, expected);
}

#[test]
fn an_unreachable_github_is_one_flag_and_every_pull_request_reads_unknown() {
    let tasks = Tasks::new();
    let state = tasks.state();
    let github = Github::Unreached(GithubFailure::new(TIMED_OUT));
    let shown = Status::new(&state, &github, PAST.parse().unwrap()).to_string();
    assert!(shown.starts_with("flags:\nunknown github: timed out\nroot\n") && shown.contains("| #13 unknown |"));
    assert!(Status::query(&state).field().contains("pr99:") && !Status::query(&state).field().contains("pr15"));
    let quiet = Tasks::reached(NOTHING_OPEN, REQUIRED);
    assert_eq!(Status::new(&State::default(), &quiet, PAST.parse().unwrap()).to_string(), "flags: none\n");
}
