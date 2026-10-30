use std::time::Duration;

use chrono::TimeDelta;
use meta_golden::Golden;
use meta_log::{LogStore as _, MemoryLog};
use meta_schema::{Actor, Block, Blocker, Claim, Event, ExternalKind, NewTask, Record, TaskLink};

use super::{Beat, Scene};
use crate::request::cli_failure::CliFailure;
use crate::request::heartbeat::Tick;
use crate::request::output::Output;

const TASK: &str = r#"{"task":"task-00000000000000c0","text":"waits"}"#;
const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const SINCE: &str = "2026-10-26T11:50:00Z";
const SHORT: Duration = Duration::from_secs(60);
const LONG: Duration = Duration::from_hours(24 * 365 * 100);
const ZERO: TimeDelta = TimeDelta::seconds(0);

struct Waiting;

impl Waiting {
    fn written(event: Event) -> Record {
        let actor: Actor = "alice".parse().unwrap();
        let fields = serde_json::to_value(Record::new(actor, event)).unwrap();
        let stamped = fields
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, value)| (name.clone(), if name == "at" { SINCE.into() } else { value.clone() }));
        serde_json::from_value(serde_json::Value::Object(stamped.collect())).unwrap()
    }

    fn log() -> MemoryLog {
        let task: NewTask = serde_json::from_str(TASK).unwrap();
        let id = task.task();
        let block = Block::new(id, Blocker::External(ExternalKind::Ci)).at_head(SHA.parse().unwrap());
        let events = [
            Event::TaskOpened(task),
            Event::TaskClaimed(Claim::new(id, None)),
            Event::TaskLinked(TaskLink::new(id, "21".parse().unwrap())),
            Event::TaskBlocked(block),
        ];
        let log = MemoryLog::default();
        let appended = events
            .into_iter()
            .map(Self::written)
            .try_for_each(|record| log.append(&mut |_| Ok::<_, CliFailure>(record.clone())).map(drop));
        appended.unwrap();
        log
    }

    fn ticked(within: Duration) -> Output {
        let git_dir = tempfile::tempdir().unwrap();
        let pulse = Beat::limited(ZERO, within);
        Tick::answer(git_dir.path(), &Self::log(), Scene::Stuck.answering(), pulse).unwrap()
    }
}

#[test]
fn a_tick_raises_a_stuck_ci_wait_under_the_policy_limit_and_stays_quiet_under_a_longer_one() {
    let ticks = [SHORT, LONG].map(Waiting::ticked);
    Golden::for_test(module_path!(), "ticks").debugged(&ticks).unwrap();
}
