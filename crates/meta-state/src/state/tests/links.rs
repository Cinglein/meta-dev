use meta_schema::{Event, PullRequest, TaskLink};

use super::{Events, Lifecycle, OPENED, OWNER, Script};
use crate::refusal::Refusal;
use crate::state::State;

const FIRST: &str = "12";
const SECOND: &str = "#13";

fn link(task: &meta_schema::NewTask, pull_request: &str) -> Event {
    Event::TaskLinked(TaskLink::new(task.task(), pull_request.parse().unwrap()))
}

#[test]
fn a_task_links_several_pull_requests_once_each_and_only_known_tasks_link() {
    let task = Events::task(vec![]);
    let ghost = Events::task(vec![]);
    let script = Script::default()
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(OWNER, link(&task, FIRST))
        .then(OWNER, link(&task, SECOND))
        .then(OWNER, link(&task, FIRST))
        .then(OWNER, link(&ghost, FIRST));
    let state = State::fold(&script.records);
    let linked: Vec<String> = state.tasks().iter().flat_map(|task| task.pull_requests().iter().map(ToString::to_string)).collect();
    assert_eq!(linked, ["#12", "#13"]);
    let expected = [Refusal::AlreadyLinked(FIRST.parse::<PullRequest>().unwrap()), Refusal::UnknownTask(ghost.task())];
    let refused = script.refused();
    assert!(refused.iter().zip(&expected).all(|(found, refusal)| found.ends_with(&refusal.to_string())));
    assert_eq!(refused.len(), expected.len());
}
