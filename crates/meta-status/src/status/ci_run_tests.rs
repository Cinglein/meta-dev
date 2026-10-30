use std::time::Duration;

use meta_schema::{Actor, Block, Blocker, Claim, Event, ExternalKind, NewTask, Record, TaskLink};
use meta_state::State;

use super::Status;
use super::tests::{NOW, Tasks};
use crate::flags::Flags;

const TASK: &str = r#"{"task":"task-00000000000000c0","text":"waits"}"#;
const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "fedcba9876543210fedcba9876543210fedcba98";
const WITHIN: Duration = Duration::from_secs(300);
const OLD: &str = "2026-10-26T11:50:00Z";
const FRESH: &str = "2026-10-26T11:57:00Z";
const RAN: &str = r#"{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[{"name":"ci","result":"PENDING"}]}}"#;
const NONE: &str = "null";
const REQUIRED: &str = r#"["ci"]"#;
const TRUNK: &str = r#""trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}"#;
const REASON: &str = "no CI run for 0123456789abcdef0123456789abcdef01234567 after 5 min; GitHub merge state=unknown";

struct Scene {
    kind: ExternalKind,
    head: &'static str,
    rollup: &'static str,
    committed: &'static str,
    within: Option<Duration>,
}

impl Scene {
    const STUCK: Self = Self { kind: ExternalKind::Ci, head: SHA, rollup: NONE, committed: OLD, within: Some(WITHIN) };

    fn reply(&self) -> String {
        format!(
            r#"{{"data":{{"open":{{"pullRequests":{{"pageInfo":{{"hasNextPage":false}},"nodes":[]}}}},"linked":{{"pr21":{{"number":21,"state":"OPEN","baseRefName":"main","statusCheckRollup":{rollup},"mergeStateStatus":"UNKNOWN","headRefOid":"{SHA}","headRef":{{"target":{{"committedDate":"{committed}"}}}}}}}},{TRUNK}}}}}"#,
            rollup = self.rollup,
            committed = self.committed
        )
    }

    fn state(&self) -> State {
        let actor: Actor = "alice".parse().unwrap();
        let task: NewTask = serde_json::from_str(TASK).unwrap();
        let id = task.task();
        let block = Block::new(id, Blocker::External(self.kind)).at_head(self.head.parse().unwrap());
        let events = [
            Event::TaskOpened(task),
            Event::TaskClaimed(Claim::new(id, None)),
            Event::TaskLinked(TaskLink::new(id, "21".parse().unwrap())),
            Event::TaskBlocked(block),
        ];
        let records: Vec<Record> = events.into_iter().map(|event| Record::new(actor.clone(), event)).collect();
        State::fold(&records)
    }

    fn flags(&self) -> Flags {
        let state = self.state();
        let github = Tasks::reached(&self.reply(), REQUIRED);
        let status = Status::new(&state, &github, NOW.parse().unwrap());
        self.within.map_or(status, |within| status.ci_run_within(within)).flags()
    }
}

#[test]
fn a_wait_on_ci_flags_a_head_with_no_run_five_minutes_after_its_push_and_says_why() {
    let flagged = Scene::STUCK.flags().to_string();
    assert_eq!(flagged, format!("waiting #21 task-00000000000000c0: {REASON}\n"));
}

#[test]
fn a_wait_is_not_flagged_when_a_run_exists_the_push_is_fresh_the_wait_is_not_on_ci_or_no_limit_is_set() {
    let quiet = [
        Scene { rollup: RAN, ..Scene::STUCK },
        Scene { committed: FRESH, ..Scene::STUCK },
        Scene { kind: ExternalKind::Review, ..Scene::STUCK },
        Scene { within: None, ..Scene::STUCK },
        Scene { head: OTHER, ..Scene::STUCK },
    ];
    assert_eq!(quiet.map(|scene| scene.flags()), [(); 5].map(|()| Flags::default()));
}
