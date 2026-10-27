use meta_pull_query::LiveReply;
use meta_schema::{Actor, Event, Id, NewTask, Record, Task, TaskLink};
use meta_state::State;

use super::Status;
use super::tests::{NOW, PAST};
use crate::github::Github;
use crate::github_failure::GithubFailure;

const REPLY: &str = r#"{"data":{"open":{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}},"linked":{
"pr1":{"number":1,"state":"MERGED","baseRefName":"main","statusCheckRollup":null},
"pr2":{"number":2,"state":"OPEN","baseRefName":"main","statusCheckRollup":null},
"pr3":{"number":3,"state":"CLOSED","baseRefName":"main","statusCheckRollup":null},
"pr99":null}}}"#;
const LINKS: [&[&str]; 8] = [&["1"], &["2"], &["3"], &["2"], &[], &["99"], &["3", "1"], &["1", "2"]];
const SHOWN: [&str; 9] = [
    "pr #1 merged",
    "pr #2 open",
    "pr #3 closed",
    "stalled: past due 2026-10-01 00:00Z, no holder",
    "no pr linked",
    "pr #99 unknown",
    "pr #1 merged",
    "pr #2 open",
    "task not in the log",
];

const UNKNOWN: &str = "pr #1 unknown";
const QUERIED: [&str; 2] = ["pr1:", "pr3:"];
const UNQUERIED: &str = "pr2:";

struct Board {
    tasks: Vec<Id<Task>>,
    state: State,
}

impl Board {
    fn new() -> Self {
        let actor: Actor = "alice".parse().unwrap();
        let opened: Vec<NewTask> = LINKS
            .iter()
            .zip(0..)
            .map(|(_, at)| {
                let task = NewTask::new("features-and-maintenance".parse().unwrap(), "x".parse().unwrap(), vec![]);
                if at == 3 { task.due_by(PAST.parse().unwrap()) } else { task }
            })
            .collect();
        let links = opened.iter().zip(LINKS).flat_map(|(task, numbers)| {
            numbers.iter().map(|number| Event::TaskLinked(TaskLink::new(task.task(), number.parse().unwrap())))
        });
        let events = opened.iter().cloned().map(Event::TaskOpened).chain(links);
        let records: Vec<Record> = events.map(|event| Record::new(actor.clone(), event)).collect();
        let tasks = opened.iter().map(NewTask::task).chain([Id::generate()]).collect();
        Self { tasks, state: State::fold(&records) }
    }

    fn shown(&self, github: &Github) -> Vec<String> {
        let status = Status::new(&self.state, github, NOW.parse().unwrap());
        self.tasks.iter().map(|task| status.lifetime(*task).to_string()).collect()
    }
}

#[test]
fn a_task_ends_when_every_linked_pull_request_is_merged_or_closed_or_it_stalls_and_unknown_never_ends() {
    let board = Board::new();
    let pulls = serde_json::from_str::<LiveReply>(REPLY).unwrap().answered().unwrap();
    let rules = [(serde_json::from_str(r#""main""#).unwrap(), serde_json::from_str("[]").unwrap())];
    let reached = Github::Reached(pulls.ruled(rules.into_iter().collect()).unwrap());
    assert_eq!(board.shown(&reached), SHOWN);
    let unreached = board.shown(&Github::Unreached(GithubFailure::new("down")));
    let queried = Status::linked_query(&board.state, &board.tasks[6..7]).field();
    assert!(
        unreached.first().map(String::as_str) == Some(UNKNOWN) && QUERIED.iter().all(|pull| queried.contains(pull))
    );
    assert!(!queried.contains(UNQUERIED), "{queried}");
}
