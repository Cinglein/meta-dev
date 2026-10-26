use meta_schema::Record;

use super::Summary;
use crate::state::State;

const LOG: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"ConcernIdentified","concern":"concern-00000000000000c1","text":"slow ci","tags":["acceptance-candidate"]}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"ConcernIdentified","concern":"concern-00000000000000c2","text":"flaky"}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"IssueReported","issue":"issue-00000000000000a1","text":"crash"}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"IssueReported","issue":"issue-00000000000000a2","text":"typo"}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000b1","text":"deflake","addresses":["concern-00000000000000c2"]}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000b2","text":"triage","addresses":["issue-00000000000000a1"]}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000b3","text":"fix crash"}
{"at":"2026-10-25T00:00:00Z","actor":"alice","kind":"TaskClaimed","task":"task-00000000000000b1"}
{"at":"2026-10-25T00:00:00Z","actor":"alice","kind":"TaskDone","task":"task-00000000000000b1"}
{"at":"2026-10-25T00:00:00Z","actor":"bob","kind":"TaskClaimed","task":"task-00000000000000b3"}
{"at":"2026-10-25T00:00:00Z","actor":"carol","kind":"TaskClaimed","task":"task-00000000000000b3","from":"alice"}
{"at":"2026-10-25T00:00:00Z","actor":"carol","kind":"TaskDone","task":"task-00000000000000b3"}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"EntryCorrected","entry":"issue-00000000000000a1","text":"crash on start"}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"EntryCorrected","entry":"concern-00000000000000c1","text":"ci is slow"}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"EntryRetired","entry":"issue-00000000000000a2","text":"dup"}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"EntryCorrected","entry":"task-00000000000000b9","text":"x"}
{"at":"2026-10-25T00:00:00Z","actor":"owner","kind":"TaskOpened","task":"task-00000000000000b4","text":"speed up","division":"performance-and-cost/ci","effort":50000,"due":"2026-11-01T00:00:00Z"}"#;

const SUMMARY: &str = "Tasks: 3 unfinished, 1 done, 0 retired
  [assigned] task-00000000000000b3 fix crash — opened by owner 2026-10-25 00:00Z — assigned bob 2026-10-25 00:00Z
  [open] task-00000000000000b2 triage >issue-00000000000000a1 — opened by owner 2026-10-25 00:00Z
  [open] task-00000000000000b4 speed up @performance-and-cost/ci effort 50000 tokens due 2026-11-01 00:00Z — opened by owner 2026-10-25 00:00Z
Issues: 1 open
  issue-00000000000000a1 crash on start — owner 2026-10-25 00:00Z
Concerns: 1 open, 1 #acceptance-candidate (meta list concerns)
Refused: 2026-10-25 00:00Z carol TaskClaimed task-00000000000000b3 from alice: alice is not assigned to the task
Refused: 2026-10-25 00:00Z carol TaskDone task-00000000000000b3: carol is not assigned to the task
Refused: 2026-10-25 00:00Z owner EntryCorrected task-00000000000000b9: x: there is no entry task-00000000000000b9
";

const CONCERNS: [&str; 2] = [
    "concern-00000000000000c1 ci is slow #acceptance-candidate — owner 2026-10-25 00:00Z",
    "concern-00000000000000c2 flaky — owner 2026-10-25 00:00Z",
];

const CLOSED: [Option<()>; 4] = [Some(()), None, None, None];

#[test]
fn the_summary_shows_what_is_left_after_work_corrections_and_refusals() {
    let records: Vec<Record> = LOG.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    let state = State::fold(&records);
    assert_eq!(Summary::new(&state).to_string(), SUMMARY);
    assert_eq!(state.concerns().iter().map(ToString::to_string).collect::<Vec<_>>(), CONCERNS);
    assert_eq!(state.tasks().iter().map(|task| state.closed(task.entry())).collect::<Vec<_>>(), CLOSED);
}
