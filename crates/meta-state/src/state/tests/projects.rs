use meta_schema::Event;

use super::{ALICE, OWNER, Script};

const META: &str = "meta";
const PROPOSE: &str = r#"{"kind":"ProjectProposed","project":"project-00000000000000a1","text":"p","division":"architecture-and-planning"}"#;
const ACCEPT: &str = r#"{"kind":"ProjectAccepted","project":"project-00000000000000a1","root":"state","rows":[{"budget":"b","measure":"m","target":{"count":0}}],"spans":["fs"],"enforcement":[{"step":"test"}]}"#;
const INSTRUMENTED: &str = r#"{"kind":"ProjectObserved","project":"project-00000000000000a1","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":"instrumented"}"#;
const ON_TARGET: &str = r#"{"kind":"ProjectObserved","project":"project-00000000000000a1","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":"on_target"}"#;
const OFF_TARGET: &str = r#"{"kind":"ProjectObserved","project":"project-00000000000000a1","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":{"off_target":"b"}}"#;
const UNINSTRUMENTED: &str = r#"{"kind":"ProjectObserved","project":"project-00000000000000a1","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":{"uninstrumented":{"unmeasured":"b"}}}"#;
const NOT_GATING: &str = r#"{"kind":"ProjectObserved","project":"project-00000000000000a1","main":"6c1ea3b4d9f1f48d98517b891f4283d6f0e55b94","run":7,"verdict":{"not_gating":{"step":"test"}}}"#;
const INSTRUMENTS: &str =
    r#"{"kind":"TaskPurposed","task":"task-00000000000000c1","purpose":{"instruments":"project-00000000000000a1"}}"#;
const SERVES: &str =
    r#"{"kind":"TaskPurposed","task":"task-00000000000000c1","purpose":{"serves":"project-00000000000000a1"}}"#;
const CAUSED_IN: &str = r#"{"kind":"TaskPurposed","task":"task-00000000000000c1","purpose":{"caused_by":{"cause":{"pull":7},"project":"project-00000000000000a1"}}}"#;
const SERVES_GHOST: &str =
    r#"{"kind":"TaskPurposed","task":"task-00000000000000c1","purpose":{"serves":"project-00000000000000b2"}}"#;
const OPENED_SERVING: &str = r#"{"kind":"TaskOpened","task":"task-00000000000000c4","text":"x","purpose":{"serves":"project-00000000000000a1"}}"#;
const REPURPOSED_OPENED: &str =
    r#"{"kind":"TaskPurposed","task":"task-00000000000000c2","purpose":{"serves":"project-00000000000000a1"}}"#;
const REPURPOSED: &str =
    r#"{"kind":"TaskPurposed","task":"task-00000000000000c3","purpose":{"caused_by":{"cause":{"pull":8}}}}"#;
const OPENED_CAUSED: &str =
    r#"{"kind":"TaskOpened","task":"task-00000000000000c2","text":"u","purpose":{"caused_by":{"cause":{"pull":7}}}}"#;
const OPENED_PLAIN: &str = r#"{"kind":"TaskOpened","task":"task-00000000000000c3","text":"w"}"#;
const PURPOSED_CAUSED: &str =
    r#"{"kind":"TaskPurposed","task":"task-00000000000000c3","purpose":{"caused_by":{"cause":{"pull":7}}}}"#;
const OPENED_TASK: &str = r#"{"kind":"TaskOpened","task":"task-00000000000000c1","text":"t"}"#;
const HISTORY: [Step; 9] = [
    Step::written(ALICE, OPENED_CAUSED),
    Step::written(ALICE, OPENED_PLAIN),
    Step::written(ALICE, PURPOSED_CAUSED),
    Step::written(ALICE, OPENED_TASK),
    Step::written(ALICE, PROPOSE),
    Step::written(OWNER, ACCEPT),
    Step::written(META, INSTRUMENTED),
    Step::written(META, ON_TARGET),
    Step::written(META, OFF_TARGET),
];
const UNPROPOSED: &[Step] = HISTORY.split_at(4).0;
const PROPOSED: &[Step] = HISTORY.split_at(5).0;
const ACCEPTED: &[Step] = HISTORY.split_at(6).0;
const MEASURED: &[Step] = HISTORY.split_at(7).0;
const DELIVERED: &[Step] = HISTORY.split_at(8).0;
const REGRESSED: &[Step] = &HISTORY;
const REFUSED: [(&[Step], Step); 25] = [
    (UNPROPOSED, Step::written(ALICE, SERVES)),
    (PROPOSED, Step::written(ALICE, PROPOSE)),
    (PROPOSED, Step::written(ALICE, ACCEPT)),
    (PROPOSED, Step::written(META, INSTRUMENTED)),
    (PROPOSED, Step::written(ALICE, INSTRUMENTS)),
    (PROPOSED, Step::written(ALICE, SERVES)),
    (PROPOSED, Step::written(ALICE, CAUSED_IN)),
    (PROPOSED, Step::written(ALICE, OPENED_SERVING)),
    (PROPOSED, Step::written(ALICE, SERVES_GHOST)),
    (ACCEPTED, Step::written(META, UNINSTRUMENTED)),
    (ACCEPTED, Step::written(META, ON_TARGET)),
    (ACCEPTED, Step::written(ALICE, REPURPOSED_OPENED)),
    (ACCEPTED, Step::written(ALICE, REPURPOSED)),
    (MEASURED, Step::written(META, INSTRUMENTED)),
    (MEASURED, Step::written(META, OFF_TARGET)),
    (MEASURED, Step::written(META, NOT_GATING)),
    (DELIVERED, Step::written(OWNER, ACCEPT)),
    (DELIVERED, Step::written(META, ON_TARGET)),
    (DELIVERED, Step::written(META, INSTRUMENTED)),
    (DELIVERED, Step::written(ALICE, INSTRUMENTS)),
    (DELIVERED, Step::written(ALICE, SERVES)),
    (REGRESSED, Step::written(OWNER, ACCEPT)),
    (REGRESSED, Step::written(META, OFF_TARGET)),
    (REGRESSED, Step::written(ALICE, INSTRUMENTS)),
    (REGRESSED, Step::written(ALICE, SERVES)),
];

struct Step {
    actor: &'static str,
    json: &'static str,
}

impl Step {
    const fn written(actor: &'static str, json: &'static str) -> Self {
        Self { actor, json }
    }

    fn event(&self) -> Event {
        serde_json::from_str(self.json).unwrap()
    }

    fn replayed(log: &[Step]) -> Script {
        log.iter().fold(Script::default(), |script, step| script.then(step.actor, step.event()))
    }
}

#[test]
fn every_refused_project_transition_and_attach_is_refused() {
    let accepted =
        REFUSED.iter().enumerate().filter(|(_, (log, step))| Step::replayed(log).vet(step.actor, step.event()).is_ok());
    assert_eq!(accepted.map(|(row, _)| row).collect::<Vec<usize>>(), Vec::<usize>::new());
}
