use meta_schema::{Actor, Block, Blocker, Claim, Event, ExternalKind, NewTask, Record, TaskLink};
use meta_state::State;

use super::Status;
use super::tests::{NOW, Tasks, WITHIN};
use crate::flags::Flags;

const TASK: &str = r#"{"task":"task-00000000000000c0","text":"waits"}"#;
const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "fedcba9876543210fedcba9876543210fedcba98";
const SINCE: &str = "2026-10-26T11:50:00Z";
const AT_LIMIT: &str = "2026-10-26T11:55:00Z";
const JUST_PAST: &str = "2026-10-26T11:54:59Z";
const FRESH: &str = "2026-10-26T11:57:00Z";
const FUTURE: &str = "2026-10-26T12:30:00Z";
const RAN: &str = r#"{"contexts":{"pageInfo":{"hasNextPage":false},"nodes":[{"name":"ci","result":"PENDING"}]}}"#;
const NONE: &str = "null";
const REQUIRED: &str = r#"["ci"]"#;
const TRUNK: &str = r#""trunk":{"defaultBranchRef":{"target":{"history":{"pageInfo":{"hasNextPage":true},"nodes":[{"checkSuites":{"pageInfo":{"hasNextPage":false},"nodes":[{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"ci"}}},{"conclusion":"SUCCESS","workflowRun":{"workflow":{"name":"mutants"}}}]}}]}}}}"#;

struct Scene {
    kind: ExternalKind,
    head: &'static str,
    rollup: &'static str,
    since: &'static str,
}

impl Scene {
    const STUCK: Self = Self { kind: ExternalKind::Ci, head: SHA, rollup: NONE, since: SINCE };

    fn reply(&self) -> String {
        format!(
            r#"{{"data":{{"open":{{"pullRequests":{{"pageInfo":{{"hasNextPage":false}},"nodes":[]}}}},"linked":{{"pr21":{{"number":21,"state":"OPEN","baseRefName":"main","statusCheckRollup":{rollup},"mergeStateStatus":"UNKNOWN","headRefOid":"{SHA}"}}}},{TRUNK}}}}}"#,
            rollup = self.rollup
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
        let written = events.into_iter().map(|event| self.written(&actor, event));
        State::fold(&written.collect::<Vec<_>>())
    }

    fn written(&self, actor: &Actor, event: Event) -> Record {
        let fields = serde_json::to_value(Record::new(actor.clone(), event)).unwrap();
        let stamped = fields
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, value)| (name.clone(), if name == "at" { self.since.into() } else { value.clone() }));
        serde_json::from_value(serde_json::Value::Object(stamped.collect())).unwrap()
    }

    fn flags(&self) -> Flags {
        let state = self.state();
        let github = Tasks::reached(&self.reply(), REQUIRED);
        Status::new(&state, &github, NOW.parse().unwrap(), WITHIN).flags()
    }
}

#[test]
fn a_wait_on_ci_flags_a_head_with_no_run_once_the_wait_is_older_than_the_limit_and_says_when_it_was_due() {
    let due = |at: &'static str| Scene { since: at, ..Scene::STUCK }.flags().to_string();
    assert_eq!(due(SINCE), format!("waiting #21 task-00000000000000c0: no CI run for {SHA}, due 2026-10-26 11:55Z\n"));
    assert_eq!(
        due(JUST_PAST),
        format!("waiting #21 task-00000000000000c0: no CI run for {SHA}, due 2026-10-26 11:59Z\n")
    );
}

#[test]
fn a_wait_is_not_flagged_at_the_limit_with_a_run_the_wrong_kind_a_moved_head_or_a_wait_from_the_future() {
    let quiet = [
        Scene { since: AT_LIMIT, ..Scene::STUCK },
        Scene { rollup: RAN, ..Scene::STUCK },
        Scene { since: FRESH, ..Scene::STUCK },
        Scene { since: FUTURE, ..Scene::STUCK },
        Scene { kind: ExternalKind::Review, ..Scene::STUCK },
        Scene { head: OTHER, ..Scene::STUCK },
    ];
    assert_eq!(quiet.map(|scene| scene.flags()), [(); 6].map(|()| Flags::default()));
}
