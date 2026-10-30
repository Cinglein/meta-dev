use meta_schema::{Actor, Claim, Event, Id, NewTask, Record, Task, TaskLink};
use meta_state::State;

use super::Status;
use super::tests::{NOW, PAST, Tasks, WITHIN};
use crate::github::Github;
use crate::github_failure::GithubFailure;

const DUE: &str = "due 2026-10-01 00:00Z";
const TRUNK_GREEN: &str = r#""trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}"#;
const TRUNK_RED: &str = r#""trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"FAILURE","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":null,"workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}"#;
const PULLS: &str = r#""open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}},"linked":{
"pr1":{"number":1,"state":"MERGED","baseRefName":"main","statusCheckRollup":null},
"pr2":{"number":2,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},
"pr3":{"number":3,"state":"CLOSED","baseRefName":"main","statusCheckRollup":null},
"pr99":null}"#;
const NO_PULLS: &str = r#""open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}}"#;
const CASES: [(bool, &[&str], bool); 8] = [
    (true, &["1"], false),
    (true, &["3", "1"], false),
    (true, &["1", "2"], false),
    (true, &[], true),
    (false, &["1"], false),
    (true, &["1", "99"], false),
    (true, &["2"], false),
    (true, &["3"], true),
];
const DONE: &str = "held, every linked pr merged or closed";

struct Board {
    tasks: Vec<Id<Task>>,
    state: State,
}

impl Board {
    fn new() -> Self {
        let actor: Actor = "alice".parse().unwrap();
        let opened: Vec<NewTask> = CASES
            .iter()
            .map(|(_, _, late)| {
                let task = NewTask::new("features-and-maintenance".parse().unwrap(), "x".parse().unwrap(), vec![]);
                if *late { task.due_by(PAST.parse().unwrap()) } else { task }
            })
            .collect();
        let held = opened.iter().zip(CASES).filter(|(_, case)| case.0);
        let claims = held.map(|(task, _)| Event::TaskClaimed(Claim::new(task.task(), None)));
        let links = opened.iter().zip(CASES).flat_map(|(task, case)| {
            case.1.iter().map(|number| Event::TaskLinked(TaskLink::new(task.task(), number.parse().unwrap())))
        });
        let events = opened.iter().cloned().map(Event::TaskOpened).chain(claims).chain(links);
        let records: Vec<Record> = events.map(|event| Record::new(actor.clone(), event)).collect();
        Self { tasks: opened.iter().map(NewTask::task).collect(), state: State::fold(&records) }
    }

    fn reached(reply: &str) -> Github {
        Tasks::reached(reply, "[]")
    }

    fn flags(&self, github: &Github) -> String {
        Status::new(&self.state, github, NOW.parse().unwrap(), WITHIN).flags().to_string()
    }
}

#[test]
fn a_held_task_whose_linked_pull_requests_are_all_merged_or_closed_is_flagged_done() {
    let board = Board::new();
    let github = Board::reached(&format!(r#"{{"data":{{{PULLS},{TRUNK_GREEN}}}}}"#));
    let [first, second, _, third, _, fifth, _, last] = board.tasks[..] else { unreachable!() };
    let expected = format!(
        "done {first}: {DONE}\ndone {second}: {DONE}\npast due {third}: {DUE}\nunknown #99 {fifth}: not on github\npast due {last}: {DUE}\ndone {last}: {DONE}\n"
    );
    assert_eq!(board.flags(&github), expected);
}

#[test]
fn an_unreachable_github_never_reads_a_task_as_done() {
    let board = Board::new();
    let flags = board.flags(&Github::Unreached(GithubFailure::new("down")));
    assert_eq!(flags.lines().filter(|line| line.starts_with("done ")).count(), 0);
    assert!(flags.starts_with("unknown github: down\npast due "));
}

#[test]
fn a_red_default_branch_run_is_flagged_and_an_unseen_one_reads_unknown_github() {
    let state = State::default();
    let red = Board::reached(&format!(r#"{{"data":{{{NO_PULLS},{TRUNK_RED}}}}}"#));
    let expected =
        "trunk red ci: latest completed run failed\nunknown github: no completed mutants run on the default branch\n";
    assert_eq!(Status::new(&state, &red, NOW.parse().unwrap(), WITHIN).flags().to_string(), expected);
    let green = Board::reached(&format!(r#"{{"data":{{{NO_PULLS},{TRUNK_GREEN}}}}}"#));
    assert_eq!(Status::new(&state, &green, NOW.parse().unwrap(), WITHIN).flags().raised(), None);
}
