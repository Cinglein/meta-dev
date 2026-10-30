use std::io::{Error, ErrorKind, Result};

use meta_golden::Golden;
use meta_schema::{Claim, Entry, EntryNote, Event, Id, Record, TaskRef};
use meta_state::State;

use super::{Brief, BriefOptions, BriefRequest, Rules};
use crate::request::cli_failure::CliFailure;
use crate::request::parsed::Parsed;

const ID: &str = "task-0123456789abcdef";
const OTHER: &str = "task-fedcba9876543210";
const TASK: &str = r#"{"task":"task-0123456789abcdef","text":"Print a brief","division":"features-and-maintenance","addresses":["concern-0123456789abcdef"],"effort":5000,"due":"2026-11-01T00:00:00Z"}"#;
const PLAIN: &str = r#"{"task":"task-0123456789abcdef","text":"Print a brief","division":"features-and-maintenance"}"#;
const RULES: &str = "version: 7\n- first rule\n- second rule\n";
const ACTOR: &str = "claude-fixture";
const SCRATCH: &str = "/scratch";
const REPOSITORY: &str = "Cinglein/fixture";

struct Fixture;

impl Fixture {
    fn state(after: &[Event]) -> State {
        let opened = Event::TaskOpened(serde_json::from_str(PLAIN).unwrap());
        let events = std::iter::once(opened).chain(after.iter().cloned());
        let records: Vec<Record> = events.map(|event| Record::new(ACTOR.parse().unwrap(), event)).collect();
        State::fold(&records)
    }

    fn briefed(
        request: BriefRequest,
        after: &[Event],
        rules: Result<String>,
    ) -> std::result::Result<Brief, CliFailure> {
        request.briefed(&Self::state(after), rules, || Ok(ACTOR.parse().unwrap()), REPOSITORY.parse().unwrap())
    }
}

#[test]
fn a_brief_gives_the_task_the_setup_and_the_rules_verbatim() {
    let options =
        BriefOptions::new(ACTOR.parse().unwrap(), Some(SCRATCH.parse().unwrap()), REPOSITORY.parse().unwrap());
    let brief = Brief::new(serde_json::from_str(TASK).unwrap(), RULES.parse::<Rules>().unwrap(), options);
    Golden::for_test(module_path!(), "brief").shown(&brief).unwrap();
}

#[test]
fn every_way_a_brief_can_fail_says_why() {
    let id: Id<_> = ID.parse().unwrap();
    let claimed = Event::TaskClaimed(Claim::new(id, None));
    let done = Event::TaskDone(TaskRef::new(id));
    let retired = Event::EntryRetired(EntryNote::new(Entry::Task(id), "no".parse().unwrap()));
    let missing = Err(Error::from(ErrorKind::NotFound));
    let table: [(&str, Vec<Event>, Result<String>, &str); 7] = [
        (ID, vec![], Ok("- a rule\n".into()), "version"),
        (ID, vec![], Ok("version: x\n".into()), "version"),
        (ID, vec![], Ok(String::new()), "version"),
        (ID, vec![], missing, "agent-rules.md"),
        (OTHER, vec![], Ok(RULES.into()), "unknown task"),
        (ID, vec![claimed, done], Ok(RULES.into()), "is done"),
        (ID, vec![retired], Ok(RULES.into()), "is retired"),
    ];
    let checked: std::result::Result<Vec<()>, String> = table
        .into_iter()
        .map(|(task, after, rules, expected)| {
            let said = Fixture::briefed(BriefRequest::new(task.parse().unwrap(), None, None), &after, rules);
            let said = said.unwrap_err().to_string();
            said.contains(expected).then_some(()).ok_or(format!("{expected}: {said}"))
        })
        .collect();
    checked.unwrap();
}

#[test]
fn the_actor_and_scratch_options_change_the_printed_brief() {
    let table = [
        (vec![], "as claude-fixture", "gh repo clone Cinglein/fixture ./task-"),
        (vec!["--actor", "alice"], "as alice", "META_ACTOR=alice "),
        (vec!["--scratch", "/elsewhere"], "as claude-fixture", "fixture /elsewhere/task-"),
    ];
    let checked: std::result::Result<Vec<()>, String> = table
        .into_iter()
        .map(|(options, actor, scratch)| {
            let words: Vec<String> = [ID].iter().chain(&options).map(ToString::to_string).collect();
            let request = Parsed::try_from(&words[..]).unwrap().brief().unwrap();
            let shown = Fixture::briefed(request, &[], Ok(RULES.into())).unwrap().to_string();
            (shown.contains(actor) && shown.contains(scratch)).then_some(()).ok_or(format!("{options:?}: {shown}"))
        })
        .collect();
    checked.unwrap();
}
