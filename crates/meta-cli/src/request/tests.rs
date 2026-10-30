mod fetches;
mod gc_restore;
mod heartbeat;
#[cfg(test)]
mod moving;
mod notices;
#[cfg(test)]
mod projects;
#[cfg(test)]
mod rating;
#[cfg(test)]
mod status;
mod trace;
mod usage;

use meta_log::{LogStore, MemoryLog};
use meta_severity::Scales;

use super::Request;
use super::ScalePolicy;
use super::cli_failure::{CliFailure, Invalid};

const PROGRAM: &str = "meta";
const OWNER: &str = "owner";
const ALICE: &str = "alice";
const DIVISION: &str = "features-and-maintenance";
const CONCERN: [&str; 9] = ["concern", "CI", "is", "slow", "--source", "chat", "--division", DIVISION, "--tag"];
const CONCERN_SHOWN: &str = " CI is slow @features-and-maintenance #acceptance-candidate (from chat)";
const OWN_GOAL: [&str; 2] = ["--caused-by", "1"];
const OPEN_DASHED: [&str; 8] = ["open", "--division", DIVISION, OWN_GOAL[0], OWN_GOAL[1], "--", "--dashed", "text"];
const ISSUE_PLACED: [&str; 4] = ["issue", "--division", DIVISION, "crash"];
const OPEN_PLANNED: [&str; 10] = [
    "open",
    "--division",
    "performance-and-cost/ci",
    OWN_GOAL[0],
    OWN_GOAL[1],
    "--effort",
    "5000",
    "--due",
    "2026-11-01T00:00:00Z",
    "speed",
];
const LIST_TASKS: [&str; 2] = ["list", "tasks"];
const LINK: &str = "link";
const PULL_REQUEST: &str = "12";
const PLANNED_SHOWN: &str = "speed @performance-and-cost/ci effort 5000 tokens due 2026-11-01 00:00Z caused_by #1";
const NOT_HERE: &str = "--source does not apply here\nusage: meta <command>";
const OPEN_FOR: [&str; 7] = ["open", "budget", "--division", DIVISION, OWN_GOAL[0], OWN_GOAL[1], "--addresses"];
const CLAIM: &str = "claim";
const DONE: &str = "done";
const WRITES: usize = 3;
const ID_INDEX: usize = 4;

#[derive(Default)]
struct Meta(MemoryLog);

impl Meta {
    fn answer(log: &impl LogStore, actor: &str, scales: &Scales, words: &[&str]) -> Result<String, CliFailure> {
        let args = std::iter::once(PROGRAM).chain(words.iter().copied()).map(String::from).collect::<Vec<_>>();
        let policy = ScalePolicy::from(scales.clone());
        Request::try_from(args)
            .and_then(|request| {
                request.answer_from(log, &policy, || actor.parse().map_err(CliFailure::raised::<Invalid>))
            })
            .map(|output| output.to_string())
    }

    fn run(&self, actor: &str, words: &[&str]) -> Result<String, CliFailure> {
        Self::answer(&self.0, actor, &Scales::default(), words)
    }

    fn written(&self, actor: &str, words: &[&str]) -> String {
        self.run(actor, words).unwrap_or_default().split_whitespace().nth(ID_INDEX).unwrap_or_default().to_owned()
    }
}

#[test]
fn a_task_taken_over_addresses_its_concern_when_the_new_claimant_finishes_it() {
    let meta = Meta::default();
    let concern = meta.written(OWNER, &[&CONCERN[..], &["acceptance-candidate"]].concat());
    let task = meta.written(OWNER, &[&OPEN_FOR[..], &[concern.as_str()]].concat());
    meta.run(OWNER, &[CLAIM, &task]).unwrap();
    meta.run(ALICE, &[CLAIM, &task, "--from", OWNER]).unwrap();
    meta.run(ALICE, &[DONE, &task]).unwrap();
    assert!(meta.run(OWNER, &["list", "concerns", "--all"]).unwrap().contains(&format!("{concern}{CONCERN_SHOWN}")));
    assert!(meta.run(OWNER, &["list"]).unwrap().contains("Concerns: 0 open"));
}

#[test]
fn refusals_and_bad_input_say_why_and_write_nothing() {
    let meta = Meta::default();
    let task = meta.written(OWNER, &OPEN_DASHED);
    meta.run(ALICE, &[CLAIM, &task]).unwrap();
    assert!(meta.run(OWNER, &[DONE, &task]).unwrap_err().to_string().contains("owner is not assigned to the task"));
    let complaints = [&["issue", "--verbose"][..], &["issue", "text", "--source", "chat"][..]]
        .map(|words| meta.run(OWNER, words).unwrap_err().to_string());
    assert!(complaints[0].contains("unknown option") && complaints[1].contains(NOT_HERE));
    assert_eq!(meta.0.read().unwrap().records().len(), WRITES);
}

#[test]
fn divisions_are_required_and_tasks_take_effort_due_links_and_waits() {
    let meta = Meta::default();
    assert!(meta.run(OWNER, &["issue", "text"]).unwrap_err().to_string().contains("--division"));
    assert!(meta.run(OWNER, &["open", "--division", "nowhere", "text"]).is_err());
    let issue = meta.written(OWNER, &ISSUE_PLACED);
    let task = meta.written(OWNER, &OPEN_PLANNED);
    meta.run(ALICE, &[CLAIM, &task]).unwrap();
    meta.run(ALICE, &["wait", &task, "--on", &issue]).unwrap();
    meta.run(ALICE, &[LINK, &task, PULL_REQUEST]).unwrap();
    let waiting = meta.run(OWNER, &LIST_TASKS).unwrap();
    assert!(waiting.contains(PLANNED_SHOWN) && waiting.contains(&format!("[in discussion {issue}]")));
    meta.run(ALICE, &["resume", &task]).unwrap();
    let resumed = meta.run(OWNER, &LIST_TASKS).unwrap();
    assert!(resumed.contains("[assigned] ") && resumed.contains("PR #12"));
}
