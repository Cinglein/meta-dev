use meta_golden::Golden;
use meta_pull_query::{LiveData, LiveReply};
use meta_pulls::Stop;
use meta_schema::{Claim, Event, Id, NewTask, PullRequest, Record, Task, TaskLink};
use meta_state::State;
use meta_status::GithubFailure;

use super::close_out::CloseOut;
use super::queue_run::QueueRun;
use crate::Request;
use crate::request::output::Listed;

const LINKED: &str = include_str!("../../../../../goldens/queue/linked.json");
const MAIN: &str = r#""main""#;
const REQUIRED: &str = r#"["ci-gate"]"#;
const DIVISION: &str = "architecture-and-planning";
const HOLDER: &str = "claude-holder";
const LANDED: &str = "196";
const TASK: &str = "task";
const QUEUE_WORDS: [&[&str]; 3] =
    [&["meta", "queue", "run", "7"], &["meta", "queue", "run"], &["meta", "queue", "stop", "7"]];

struct Recorded;

impl Recorded {
    fn live() -> Result<LiveData, GithubFailure> {
        let open = r#"{"pullRequests":{"pageInfo":{"hasNextPage":false},"nodes":[]}}"#;
        let reply = format!(r#"{{"data":{{"open":{open},"linked":{LINKED}}}}}"#);
        let pulls = serde_json::from_str::<LiveReply>(&reply).unwrap().answered().unwrap();
        let rules = [(serde_json::from_str(MAIN).unwrap(), serde_json::from_str(REQUIRED).unwrap())];
        pulls.ruled(rules.into_iter().collect()).ok_or_else(|| GithubFailure::new(MAIN))
    }

    fn state(task: NewTask, linked: &[&str]) -> State {
        let id = task.task();
        let links = linked.iter().map(|pull| Event::TaskLinked(TaskLink::new(id, pull.parse().unwrap())));
        let events = [Event::TaskOpened(task), Event::TaskClaimed(Claim::new(id, None))].into_iter().chain(links);
        State::fold(&events.map(|event| Record::new(HOLDER.parse().unwrap(), event)).collect::<Vec<_>>())
    }

    fn closed(linked: &[&str]) -> String {
        let task = NewTask::new(DIVISION.parse().unwrap(), TASK.parse().unwrap(), vec![]);
        let id: Id<Task> = task.task();
        let close_out = CloseOut::judged(&Self::state(task, linked), &Self::live().unwrap(), LANDED.parse().unwrap());
        close_out.to_string().replace(&id.to_string(), TASK)
    }
}

#[test]
fn a_merged_head_closes_out_its_task_as_the_holder_only_once_every_linked_pr_merged() {
    assert_eq!(Recorded::closed(&[LANDED]), "task done as claude-holder");
    assert_eq!(Recorded::closed(&[LANDED, "203"]), "task still links unmerged #203: left open");
}

#[test]
fn queue_run_needs_the_run_verb_and_at_least_one_pull_request_number() {
    let parsed = |words: &[&str]| Request::try_from(words.iter().map(ToString::to_string).collect::<Vec<_>>()).is_ok();
    assert_eq!(QUEUE_WORDS.map(parsed), [true, false, false]);
}

#[test]
fn a_run_shows_every_head_and_exits_nonzero_on_a_stop_or_a_flag() {
    let pull_request: PullRequest = LANDED.parse().unwrap();
    let outputs = [
        QueueRun::new().landed(Listed::new(TASK)).output(),
        QueueRun::new().flagged(Listed::new(TASK), pull_request).output(),
        QueueRun::new().halted(&Stop::ChecksTimeout(pull_request)).output(),
    ];
    Golden::for_test(module_path!(), "outputs").debugged(&outputs).unwrap();
}
