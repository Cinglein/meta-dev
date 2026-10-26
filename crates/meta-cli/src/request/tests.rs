mod fetches;
mod heartbeat;
mod notices;
#[cfg(test)]
mod status;
mod usage;

use meta_log::{LogStore, MemoryLog};

use super::Request;
use super::cli_failure::{CliFailure, Invalid};

const PROGRAM: &str = "meta";
const OWNER: &str = "owner";
const ALICE: &str = "alice";
const DIVISION: &str = "features-and-maintenance";
const NESTED: &str = "performance-and-cost/ci";
const CONCERN: [&str; 9] = ["concern", "CI", "is", "slow", "--source", "chat", "--division", DIVISION, "--tag"];
const CONCERN_SHOWN: &str = " CI is slow @features-and-maintenance #acceptance-candidate (from chat)";
const TAG: &str = "acceptance-candidate";
const OPEN_DASHED: [&str; 6] = ["open", "--division", DIVISION, "--", "--dashed", "text"];
const ISSUE_WITH_OPTION: [&str; 2] = ["issue", "--verbose"];
const ISSUE_SOURCED: [&str; 4] = ["issue", "text", "--source", "chat"];
const ISSUE_UNPLACED: [&str; 2] = ["issue", "text"];
const ISSUE_PLACED: [&str; 4] = ["issue", "--division", DIVISION, "crash"];
const OPEN_ELSEWHERE: [&str; 4] = ["open", "--division", "nowhere", "text"];
const OPEN_PLANNED: [&str; 8] =
    ["open", "--division", NESTED, "--effort", "5000", "--due", "2026-11-01T00:00:00Z", "speed"];
const LIST_TASKS: [&str; 2] = ["list", "tasks"];
const WAIT: &str = "wait";
const ON: &str = "--on";
const RESUME: &str = "resume";
const LINK: &str = "link";
const PULL_REQUEST: &str = "12";
const PLANNED_SHOWN: &str = "speed @performance-and-cost/ci effort 5000 tokens due 2026-11-01 00:00Z";
const DISCUSSED: &str = "[in discussion ";
const ASSIGNED: &str = "[assigned] ";
const LINKED: &str = "PR #12";
const NEEDS_DIVISION: &str = "--division";
const NOT_HERE: &str = "--source does not apply here\nusage: meta <command>";
const LIST_ALL_CONCERNS: [&str; 3] = ["list", "concerns", "--all"];
const SUMMARY: [&str; 1] = ["list"];
const OPEN_FOR: [&str; 5] = ["open", "budget", "--division", DIVISION, "--addresses"];
const CLAIM: &str = "claim";
const FROM: &str = "--from";
const DONE: &str = "done";
const NOTHING_OPEN: &str = "Concerns: 0 open";
const HELD: &str = "owner is not assigned to the task";
const UNKNOWN_OPTION: &str = "unknown option";
const WRITES: usize = 3;
const ID_INDEX: usize = 4;

#[derive(Default)]
struct Meta(MemoryLog);

impl Meta {
    fn run(&self, actor: &str, words: &[&str]) -> Result<String, CliFailure> {
        let args = std::iter::once(PROGRAM).chain(words.iter().copied()).map(String::from).collect::<Vec<_>>();
        Request::try_from(args)
            .and_then(|request| request.answer_from(&self.0, || actor.parse().map_err(CliFailure::raised::<Invalid>)))
            .map(|output| output.to_string())
    }

    fn written(&self, actor: &str, words: &[&str]) -> String {
        self.run(actor, words).unwrap_or_default().split_whitespace().nth(ID_INDEX).unwrap_or_default().to_owned()
    }
}

#[test]
fn a_task_taken_over_addresses_its_concern_when_the_new_claimant_finishes_it() {
    let meta = Meta::default();
    let concern = meta.written(OWNER, &[&CONCERN[..], &[TAG]].concat());
    let task = meta.written(OWNER, &[&OPEN_FOR[..], &[concern.as_str()]].concat());
    meta.run(OWNER, &[CLAIM, &task]).unwrap();
    meta.run(ALICE, &[CLAIM, &task, FROM, OWNER]).unwrap();
    meta.run(ALICE, &[DONE, &task]).unwrap();
    assert!(meta.run(OWNER, &LIST_ALL_CONCERNS).unwrap().contains(&format!("{concern}{CONCERN_SHOWN}")));
    assert!(meta.run(OWNER, &SUMMARY).unwrap().contains(NOTHING_OPEN));
}

#[test]
fn refusals_and_bad_input_say_why_and_write_nothing() {
    let meta = Meta::default();
    let task = meta.written(OWNER, &OPEN_DASHED);
    meta.run(ALICE, &[CLAIM, &task]).unwrap();
    assert!(meta.run(OWNER, &[DONE, &task]).unwrap_err().to_string().contains(HELD));
    let complaints =
        [&ISSUE_WITH_OPTION[..], &ISSUE_SOURCED[..]].map(|words| meta.run(OWNER, words).unwrap_err().to_string());
    assert!(complaints[0].contains(UNKNOWN_OPTION) && complaints[1].contains(NOT_HERE));
    assert_eq!(meta.0.read().unwrap().records().len(), WRITES);
}

#[test]
fn divisions_are_required_and_tasks_take_effort_due_links_and_waits() {
    let meta = Meta::default();
    assert!(meta.run(OWNER, &ISSUE_UNPLACED).unwrap_err().to_string().contains(NEEDS_DIVISION));
    assert!(meta.run(OWNER, &OPEN_ELSEWHERE).is_err());
    let issue = meta.written(OWNER, &ISSUE_PLACED);
    let task = meta.written(OWNER, &OPEN_PLANNED);
    meta.run(ALICE, &[CLAIM, &task]).unwrap();
    meta.run(ALICE, &[WAIT, &task, ON, &issue]).unwrap();
    meta.run(ALICE, &[LINK, &task, PULL_REQUEST]).unwrap();
    let waiting = meta.run(OWNER, &LIST_TASKS).unwrap();
    assert!(waiting.contains(PLANNED_SHOWN) && waiting.contains(&format!("{DISCUSSED}{issue}]")));
    meta.run(ALICE, &[RESUME, &task]).unwrap();
    let resumed = meta.run(OWNER, &LIST_TASKS).unwrap();
    assert!(resumed.contains(ASSIGNED) && resumed.contains(LINKED));
}
