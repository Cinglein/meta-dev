use meta_acceptance::{AcceptanceRow, Enforcement, Gating, Observability};

use crate::{Acceptance, Cause, Event, Purpose, Record, Root};

const LINES: [&str; 16] = [
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"SchemaVersion","version":7}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-0000000000000004","text":"x"}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"ProjectProposed","project":"project-0000000000000001","text":"x","division":"architecture-and-planning"}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[{"budget":"test-spawns","measure":"test_spawns_outside_allowance","target":{"count":0}}],"spans":["spawn git","fs"],"enforcement":[{"budget":"test-spawns"},{"step":"test"},{"hook":"pre-tool-use agent"}]}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"process","rows":[{"budget":"b","measure":"m","target":{"count":1}}],"spans":["fs"],"enforcement":[{"step":"lint"}]}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"meta","kind":"ProjectObserved","project":"project-0000000000000001","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":{"uninstrumented":{"no_span":"command claim"}}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"meta","kind":"ProjectObserved","project":"project-0000000000000001","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":{"uninstrumented":{"unmeasured":"b"}}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"meta","kind":"ProjectObserved","project":"project-0000000000000001","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":"instrumented"}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"meta","kind":"ProjectObserved","project":"project-0000000000000001","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":"on_target"}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"meta","kind":"ProjectObserved","project":"project-0000000000000001","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":{"off_target":"b"}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"meta","kind":"ProjectObserved","project":"project-0000000000000001","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":{"not_gating":{"hook":"h"}}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"TaskPurposed","task":"task-0000000000000002","purpose":{"caused_by":{"cause":{"pull":217},"project":"project-0000000000000001"}}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"TaskPurposed","task":"task-0000000000000002","purpose":{"serves":"project-0000000000000001"}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"TaskPurposed","task":"task-0000000000000002","purpose":{"caused_by":{"cause":{"task":"task-0000000000000005"}}}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-0000000000000003","text":"x","purpose":{"instruments":"project-0000000000000001"}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-0000000000000003","text":"x","purpose":{"caused_by":{"cause":{"item":"issue-0000000000000006"}}}}"#,
];
const SHOWN: [&str; 16] = [
    "SchemaVersion 7",
    "TaskOpened task-0000000000000004 x",
    "ProjectProposed project-0000000000000001 x @architecture-and-planning",
    "ProjectAccepted project-0000000000000001 state test-spawns:test_spawns_outside_allowance<=0 span spawn git span fs gate budget:test-spawns gate step:test gate hook:pre-tool-use agent",
    "ProjectAccepted project-0000000000000001 process b:m<=1 span fs gate step:lint",
    "ProjectObserved project-0000000000000001 uninstrumented no span command claim at 6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94 run 7",
    "ProjectObserved project-0000000000000001 uninstrumented unmeasured b at 6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94 run 7",
    "ProjectObserved project-0000000000000001 instrumented at 6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94 run 7",
    "ProjectObserved project-0000000000000001 on target at 6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94 run 7",
    "ProjectObserved project-0000000000000001 off target b at 6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94 run 7",
    "ProjectObserved project-0000000000000001 not gating hook:h at 6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94 run 7",
    "TaskPurposed task-0000000000000002 caused_by #217 in project-0000000000000001",
    "TaskPurposed task-0000000000000002 serves project-0000000000000001",
    "TaskPurposed task-0000000000000002 caused_by task-0000000000000005",
    "TaskOpened task-0000000000000003 x instruments project-0000000000000001",
    "TaskOpened task-0000000000000003 x caused_by issue-0000000000000006",
];
const REFUSED: [(&str, &str); 8] = [
    (
        r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"ProjectProposed","project":"project-0000000000000001","text":"x"}"#,
        "missing field `division`",
    ),
    (
        r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[],"spans":["fs"],"enforcement":[{"step":"test"}]}"#,
        "needs at least one entry",
    ),
    (
        r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[{"budget":"b","measure":"m","target":{"count":0}}],"spans":[],"enforcement":[{"step":"test"}]}"#,
        "needs at least one entry",
    ),
    (
        r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[{"budget":"b","measure":"m","target":{"count":0}}],"spans":["fs"],"enforcement":[{"hook":"h"}]}"#,
        "needs at least one budget or step",
    ),
    (
        r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[{"budget":"b","measure":"m","target":{"count":0}},{"budget":"b","measure":"n","target":{"count":0}}],"spans":["fs"],"enforcement":[{"step":"test"}]}"#,
        "budget b has two acceptance rows",
    ),
    (
        r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[{"budget":"","measure":"m","target":{"count":0}}],"spans":["fs"],"enforcement":[{"step":"test"}]}"#,
        "a budget id is not empty",
    ),
    (
        r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[{"budget":"b","measure":"Test Time","target":{"count":0}}],"spans":["fs"],"enforcement":[{"step":"test"}]}"#,
        "a measure name is lowercase",
    ),
    (
        r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"TaskPurposed","task":"task-0000000000000002","purpose":{"serves":"task-0000000000000002"}}"#,
        "expected a project id",
    ),
];

#[test]
fn project_events_and_purposes_read_show_and_write_back_unchanged() {
    let records = LINES.map(|line| serde_json::from_str::<Record>(line).unwrap());
    assert_eq!(records.each_ref().map(|record| serde_json::to_string(record).unwrap()), LINES);
    assert_eq!(records.map(|record| record.event().to_string()), SHOWN);
}

const BUILT_ROWS: [&str; 3] = ["b:m<=250ms", "c:n<=30%", "d:o<=3"];
const BUILT: &str = r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"process","rows":[{"budget":"b","measure":"m","target":{"duration":250}},{"budget":"c","measure":"n","target":{"share":30}},{"budget":"d","measure":"o","target":{"count":3}}],"spans":["fs"],"enforcement":[{"budget":"b"},{"step":"test"},{"hook":"h"}]}"#;
const UNREADABLE: [&str; 5] = ["b:m", "b<=3", "b:m<=soon", ":m<=3", "b:M<=3"];
const POINTS: [&str; 3] = ["budget:b", "step:test", "hook:h"];
const UNREADABLE_POINTS: [&str; 3] = ["budget", "other:x", "step:"];
const CAUSES: [(&str, &str); 4] = [
    ("7", "#7"),
    ("#7", "#7"),
    ("issue-0000000000000006", "issue-0000000000000006"),
    ("task-0000000000000005", "task-0000000000000005"),
];

#[test]
fn an_acceptance_built_from_option_text_is_the_one_the_log_reads() {
    let rows = BUILT_ROWS.map(|row| row.parse::<AcceptanceRow>().unwrap()).to_vec();
    let observability = Observability::named(rows, vec!["fs".parse().unwrap()]).unwrap();
    let gating = Gating::try_from(POINTS.map(|point| point.parse::<Enforcement>().unwrap()).to_vec()).unwrap();
    let built = Acceptance::new("project-0000000000000001".parse().unwrap(), Root::Process, observability, gating);
    let read: Record = serde_json::from_str(BUILT).unwrap();
    assert_eq!(read.event(), &Event::ProjectAccepted(built));
    assert!(UNREADABLE.iter().all(|row| row.parse::<AcceptanceRow>().is_err()));
    assert!(UNREADABLE_POINTS.iter().all(|point| point.parse::<Enforcement>().is_err()));
    assert!(Gating::try_from(vec![Enforcement::Hook("h".parse().unwrap())]).is_err());
    assert!(Observability::named(vec![], vec!["fs".parse().unwrap()]).is_err());
}

#[test]
fn a_cause_is_a_pull_request_an_item_or_a_task_and_a_project_goes_only_with_a_cause() {
    let shown = CAUSES.map(|(text, _)| text.parse::<Cause>().unwrap().to_string());
    assert_eq!(shown, CAUSES.map(|(_, expected)| expected));
    assert!("project-0000000000000001".parse::<Cause>().is_err());
    let project = "project-0000000000000001".parse().unwrap();
    let caused = Purpose::CausedBy { cause: "7".parse().unwrap(), project: None };
    assert_eq!(caused.in_project(project).and_then(|purpose| purpose.project()), Some(project));
    assert_eq!(Purpose::Serves(project).in_project(project), None);
}

#[test]
fn each_invalid_project_line_is_refused_for_its_own_reason() {
    let refused = REFUSED.map(|(line, _)| serde_json::from_str::<Record>(line).unwrap_err().to_string());
    assert!(refused.iter().zip(REFUSED).all(|(found, (_, reason))| found.contains(reason)), "{refused:?}");
}
