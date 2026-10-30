use crate::Record;

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

#[test]
fn each_invalid_project_line_is_refused_for_its_own_reason() {
    let refused = REFUSED.map(|(line, _)| serde_json::from_str::<Record>(line).unwrap_err().to_string());
    assert!(refused.iter().zip(REFUSED).all(|(found, (_, reason))| found.contains(reason)), "{refused:?}");
}
