use meta_schema::{Event, NewTask, PullRequest, TaskLink};

use super::{Events, Lifecycle, OPENED, OWNER, Script};
use crate::refusal::Refusal;
use crate::state::State;

const FIRST: &str = "12";
const SECOND: &str = "#13";

struct Links;

impl Links {
    fn event(task: &NewTask, pull_request: &str) -> Event {
        Event::TaskLinked(TaskLink::new(task.task(), pull_request.parse().unwrap()))
    }
}

#[test]
fn a_task_links_several_pull_requests_once_each_and_only_known_tasks_link() {
    let task = Events::task(vec![]);
    let ghost = Events::task(vec![]);
    let script = Script::default()
        .then(OWNER, Lifecycle::event(&task, OPENED))
        .then(OWNER, Links::event(&task, FIRST))
        .then(OWNER, Links::event(&task, SECOND))
        .then(OWNER, Links::event(&task, FIRST));
    let state = State::fold(&script.records);
    let linked: Vec<PullRequest> = state.tasks().iter().flat_map(|task| task.pull_requests().to_vec()).collect();
    assert_eq!(linked, [FIRST.parse().unwrap(), SECOND.parse().unwrap()]);
    assert!(state.anomalies().is_empty() && script.vet(OWNER, Links::event(&task, SECOND)).is_ok());
    assert_eq!(script.vet(OWNER, Links::event(&ghost, FIRST)), Err(Refusal::UnknownTask(ghost.task())));
}

#[test]
fn only_a_held_task_takes_new_links_and_known_ones_are_skipped() {
    let task = Events::task(vec![]);
    let pulls: Vec<PullRequest> = [FIRST, SECOND].iter().map(|number| number.parse().unwrap()).collect();
    let unheld = Script::default().then(OWNER, Lifecycle::event(&task, OPENED)).then(OWNER, Links::event(&task, FIRST));
    assert_eq!(State::fold(&unheld.records).unlinked(task.task(), &pulls), Vec::new());
    let held = unheld.then(OWNER, Events::claim(task.task(), None));
    let fresh = State::fold(&held.records).unlinked(task.task(), &pulls);
    assert_eq!(fresh, [TaskLink::new(task.task(), SECOND.parse().unwrap())]);
}
