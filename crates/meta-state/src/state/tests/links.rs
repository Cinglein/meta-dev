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
        .then(OWNER, Links::event(&task, SECOND));
    let state = State::fold(&script.records);
    let linked: Vec<PullRequest> = state.tasks().iter().flat_map(|task| task.pull_requests().to_vec()).collect();
    assert_eq!(linked, [FIRST.parse().unwrap(), SECOND.parse().unwrap()]);
    assert_eq!(script.vet(OWNER, Links::event(&task, FIRST)), Err(Refusal::AlreadyLinked(FIRST.parse().unwrap())));
    assert_eq!(script.vet(OWNER, Links::event(&ghost, FIRST)), Err(Refusal::UnknownTask(ghost.task())));
}
