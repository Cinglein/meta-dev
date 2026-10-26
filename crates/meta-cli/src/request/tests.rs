use meta_log::{LogStore, MemoryLog};

use super::Request;
use super::cli_failure::{CliFailure, Invalid};

const PROGRAM: &str = "meta";
const OWNER: &str = "owner";
const ALICE: &str = "alice";
const CONCERN: [&str; 7] = ["concern", "CI", "is", "slow", "--source", "chat", "--tag"];
const CONCERN_SHOWN: &str = " CI is slow #acceptance-candidate (from chat)";
const TAG: &str = "acceptance-candidate";
const OPEN_DASHED: [&str; 4] = ["open", "--", "--dashed", "text"];
const ISSUE_WITH_OPTION: [&str; 2] = ["issue", "--verbose"];
const ISSUE_SOURCED: [&str; 4] = ["issue", "text", "--source", "chat"];
const NOT_HERE: &str = "--source does not apply here\nusage: meta <command>";
const LIST_ALL_CONCERNS: [&str; 3] = ["list", "concerns", "--all"];
const SUMMARY: [&str; 1] = ["list"];
const OPEN_FOR: [&str; 3] = ["open", "budget", "--addresses"];
const CLAIM: &str = "claim";
const FROM: &str = "--from";
const DONE: &str = "done";
const NOTHING_OPEN: &str = "Concerns: 0 open";
const HELD: &str = "held by alice";
const UNKNOWN_OPTION: &str = "unknown option";
const WRITES: usize = 2;
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
