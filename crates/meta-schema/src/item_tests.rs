use crate::{Block, Blocker, Event, ExternalKind, NewConcern, NewIssue, NewTask, Placement, SchemaVersion, TaskLink};

const DIVISION: &str = "performance-and-cost/ci";
const TEXT: &str = "speed up";
const TOKENS: &str = "5000";
const DUE: &str = "2026-11-01T00:00:00Z";
const LEGACY_TASK: &str = r#"{"task":"task-0000000000000001","text":"x"}"#;
const LEGACY_CONCERN: &str = r#"{"concern":"concern-0000000000000001","text":"x"}"#;
const LEGACY_ISSUE: &str = r#"{"issue":"issue-0000000000000001","text":"x"}"#;
const TASK: &str = "task-0000000000000002";
const BLOCKED: &str = "TaskBlocked task-0000000000000002 on ci";
const LINKED: &str = "TaskLinked task-0000000000000002 #7";
const MARKER: &str = "SchemaVersion 2";

#[test]
fn new_items_carry_their_division_and_tasks_their_effort_and_due_time() {
    let division = || DIVISION.parse().unwrap();
    let text = || TEXT.parse().unwrap();
    let planned =
        NewTask::new(division(), text(), vec![]).allowing(TOKENS.parse().unwrap()).due_by(DUE.parse().unwrap());
    let placements = [
        planned.placement().clone(),
        NewConcern::new(division(), text(), None, vec![]).placement().clone(),
        NewIssue::new(division(), text()).placement().clone(),
    ];
    assert!(placements.iter().all(|placement| *placement == Placement::new(division())));
    assert_eq!(planned.effort_allowance(), Some(TOKENS.parse().unwrap()));
    assert_eq!(planned.due_time(), Some(DUE.parse().unwrap()));
}

#[test]
fn old_items_stay_at_the_root_and_are_written_back_without_a_division() {
    let task: NewTask = serde_json::from_str(LEGACY_TASK).unwrap();
    let concern: NewConcern = serde_json::from_str(LEGACY_CONCERN).unwrap();
    let issue: NewIssue = serde_json::from_str(LEGACY_ISSUE).unwrap();
    let written = [
        serde_json::to_string(&task).unwrap(),
        serde_json::to_string(&concern).unwrap(),
        serde_json::to_string(&issue).unwrap(),
    ];
    assert!(written.iter().all(|line| !line.contains("division")));
    assert!(task.placement().is_root() && concern.placement().is_root() && issue.placement().is_root());
    assert_eq!((task.effort_allowance(), task.due_time()), (None, None));
}

#[test]
fn blocks_and_links_show_their_task_and_what_they_point_at() {
    let task = TASK.parse().unwrap();
    let blocked = Event::TaskBlocked(Block::new(task, Blocker::External(ExternalKind::Ci)));
    let linked = Event::TaskLinked(TaskLink::new(task, "7".parse().unwrap()));
    let marker = Event::SchemaVersion(SchemaVersion::current());
    assert_eq!([blocked.to_string(), linked.to_string(), marker.to_string()], [BLOCKED, LINKED, MARKER]);
    assert!(marker.schema_version().is_some() && blocked.schema_version().is_none());
}
