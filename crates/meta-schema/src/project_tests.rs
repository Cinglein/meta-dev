use crate::Record;

const LINES: [&str; 6] = [
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"ProjectProposed","project":"project-0000000000000001","text":"x","division":"architecture-and-planning"}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[{"budget":"test-spawns","measure":"test_spawns_outside_allowance","target":{"count":0}}],"spans":[{"spawn":"git"},"fs"],"enforcement":[{"budget":"test-spawns"},{"step":"test"},{"hook":"pre-tool-use agent"}]}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"meta","kind":"ProjectObserved","project":"project-0000000000000001","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":{"uninstrumented":{"no_span":{"command":"claim"}}}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"meta","kind":"ProjectObserved","project":"project-0000000000000001","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":{"not_gating":{"step":"lint"}}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"TaskPurposed","task":"task-0000000000000002","purpose":{"caused_by":{"cause":{"pull":217},"project":"project-0000000000000001"}}}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-0000000000000003","text":"x","purpose":{"instruments":"project-0000000000000001"}}"#,
];
const SHOWN: [&str; 6] = [
    "ProjectProposed project-0000000000000001 x @architecture-and-planning",
    "ProjectAccepted project-0000000000000001 state test-spawns:test_spawns_outside_allowance<=0 span spawn git span fs gate budget:test-spawns gate step:test gate hook:pre-tool-use agent",
    "ProjectObserved project-0000000000000001 uninstrumented no span command claim at 6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94 run 7",
    "ProjectObserved project-0000000000000001 not gating step:lint at 6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94 run 7",
    "TaskPurposed task-0000000000000002 caused_by #217 in project-0000000000000001",
    "TaskOpened task-0000000000000003 x instruments project-0000000000000001",
];
const REFUSED: [&str; 5] = [
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"ProjectProposed","project":"project-0000000000000001","text":"x"}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[],"spans":["fs"],"enforcement":[{"step":"test"}]}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[{"budget":"b","measure":"m","target":{"count":0}}],"spans":[],"enforcement":[{"step":"test"}]}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"owner","kind":"ProjectAccepted","project":"project-0000000000000001","root":"state","rows":[{"budget":"b","measure":"m","target":{"count":0}}],"spans":["fs"],"enforcement":[{"hook":"h"}]}"#,
    r#"{"at":"2026-10-30T00:00:00Z","actor":"a","kind":"TaskPurposed","task":"task-0000000000000002","purpose":{"serves":"task-0000000000000002"}}"#,
];

#[test]
fn project_events_and_purposes_read_show_and_write_back_unchanged() {
    let records = LINES.map(|line| serde_json::from_str::<Record>(line).unwrap());
    assert_eq!(records.each_ref().map(|record| serde_json::to_string(record).unwrap()), LINES);
    assert_eq!(records.map(|record| record.event().to_string()), SHOWN);
}

#[test]
fn a_project_without_a_division_rows_spans_or_a_ci_gate_and_a_purpose_naming_a_task_are_refused() {
    assert!(REFUSED.iter().all(|line| serde_json::from_str::<Record>(line).is_err()));
}
