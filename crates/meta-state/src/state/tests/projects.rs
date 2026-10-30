use meta_schema::Event;

use super::{ALICE, OWNER, Script};
use crate::project_stage::ProjectStage;
use crate::refusal::Refusal;
use crate::state::State;

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
const CLAIM: &str = r#"{"kind":"TaskClaimed","task":"task-00000000000000c1"}"#;
const RELEASE: &str = r#"{"kind":"TaskReleased","task":"task-00000000000000c1"}"#;
const DONE: &str = r#"{"kind":"TaskDone","task":"task-00000000000000c1"}"#;
const RETIRE: &str = r#"{"kind":"EntryRetired","entry":"task-00000000000000c1","text":"dropped"}"#;
const PROJECT: &str = "project-00000000000000a1";
const GHOST: &str = "project-00000000000000b2";
const ON_PROPOSED: Refusal = Refusal::Unattachable(ProjectStage::Proposed);
const ON_DELIVERED: Refusal = Refusal::Unattachable(ProjectStage::Delivered);
const ON_REGRESSED: Refusal = Refusal::Unattachable(ProjectStage::Regressed);
const REFUSED: [(&[Step], Step, Refusal); 25] = [
    (PROPOSED, Step::written(ALICE, PROPOSE), Refusal::DuplicateId),
    (PROPOSED, Step::written(ALICE, ACCEPT), Refusal::NotOwner),
    (PROPOSED, Step::written(META, INSTRUMENTED), Refusal::ProjectIs(ProjectStage::Proposed)),
    (PROPOSED, Step::written(ALICE, INSTRUMENTS), ON_PROPOSED),
    (PROPOSED, Step::written(ALICE, SERVES), ON_PROPOSED),
    (PROPOSED, Step::written(ALICE, CAUSED_IN), ON_PROPOSED),
    (PROPOSED, Step::written(ALICE, OPENED_SERVING), ON_PROPOSED),
    (ACCEPTED, Step::written(META, UNINSTRUMENTED), Refusal::Unchanged),
    (ACCEPTED, Step::written(META, ON_TARGET), Refusal::NotInstrumented),
    (ACCEPTED, Step::written(ALICE, INSTRUMENTED), Refusal::NotObserver),
    (ACCEPTED, Step::written(OWNER, INSTRUMENTED), Refusal::NotObserver),
    (ACCEPTED, Step::written(ALICE, REPURPOSED_OPENED), Refusal::AlreadyPurposed),
    (ACCEPTED, Step::written(ALICE, REPURPOSED), Refusal::AlreadyPurposed),
    (MEASURED, Step::written(META, INSTRUMENTED), Refusal::Unchanged),
    (MEASURED, Step::written(META, OFF_TARGET), Refusal::Unchanged),
    (MEASURED, Step::written(META, NOT_GATING), Refusal::Unchanged),
    (DELIVERED, Step::written(OWNER, ACCEPT), Refusal::ProjectIs(ProjectStage::Delivered)),
    (DELIVERED, Step::written(META, ON_TARGET), Refusal::Unchanged),
    (DELIVERED, Step::written(META, INSTRUMENTED), Refusal::Unchanged),
    (DELIVERED, Step::written(ALICE, INSTRUMENTS), ON_DELIVERED),
    (DELIVERED, Step::written(ALICE, SERVES), ON_DELIVERED),
    (REGRESSED, Step::written(OWNER, ACCEPT), Refusal::ProjectIs(ProjectStage::Regressed)),
    (REGRESSED, Step::written(META, OFF_TARGET), Refusal::Unchanged),
    (REGRESSED, Step::written(ALICE, INSTRUMENTS), ON_REGRESSED),
    (REGRESSED, Step::written(ALICE, SERVES), ON_REGRESSED),
];
const SERVING: [Step; 2] = [Step::written(ALICE, SERVES), Step::written(ALICE, CLAIM)];
const RELEASED: [Step; 3] = [SERVING[0], SERVING[1], Step::written(ALICE, RELEASE)];
const FINISHED: [Step; 3] = [SERVING[0], SERVING[1], Step::written(ALICE, DONE)];
const RETIRED: [Step; 3] = [SERVING[0], SERVING[1], Step::written(OWNER, RETIRE)];
const STAGES: [(&[Step], &[Step], ProjectStage); 16] = [
    (PROPOSED, &[], ProjectStage::Proposed),
    (ACCEPTED, &[], ProjectStage::Accepted),
    (MEASURED, &[], ProjectStage::Instrumented),
    (DELIVERED, &[], ProjectStage::Delivered),
    (REGRESSED, &[], ProjectStage::Regressed),
    (MEASURED, &[Step::written(ALICE, SERVES)], ProjectStage::Instrumented),
    (MEASURED, &SERVING, ProjectStage::Active),
    (MEASURED, &RELEASED, ProjectStage::Instrumented),
    (MEASURED, &FINISHED, ProjectStage::Instrumented),
    (MEASURED, &RETIRED, ProjectStage::Instrumented),
    (MEASURED, &[Step::written(META, UNINSTRUMENTED)], ProjectStage::Accepted),
    (MEASURED, &[Step::written(OWNER, ACCEPT)], ProjectStage::Accepted),
    (REGRESSED, &[Step::written(META, ON_TARGET)], ProjectStage::Delivered),
    (DELIVERED, &[Step::written(META, NOT_GATING)], ProjectStage::Regressed),
    (DELIVERED, &[Step::written(META, UNINSTRUMENTED)], ProjectStage::Regressed),
    (REGRESSED, &[Step::written(ALICE, CAUSED_IN)], ProjectStage::Regressed),
];

#[derive(Clone, Copy)]
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
fn every_refused_project_transition_and_attach_names_its_refusal() {
    let found: Vec<Result<(), Refusal>> =
        REFUSED.iter().map(|(log, step, _)| Step::replayed(log).vet(step.actor, step.event())).collect();
    let expected: Vec<Result<(), Refusal>> = REFUSED.iter().map(|(.., refusal)| Err(refusal.clone())).collect();
    assert_eq!(found, expected);
}

#[test]
fn a_purpose_naming_an_unknown_project_is_refused() {
    let unproposed = Step::replayed(UNPROPOSED).vet(ALICE, Step::written(ALICE, SERVES).event());
    let ghost = Step::replayed(PROPOSED).vet(ALICE, Step::written(ALICE, SERVES_GHOST).event());
    let refused = [PROJECT, GHOST].map(|project| Err(Refusal::UnknownProject(project.parse().unwrap())));
    assert_eq!([unproposed, ghost], refused);
}

#[test]
fn the_fold_reaches_each_stage_and_active_needs_a_held_serving_task() {
    let folded: Vec<(Option<ProjectStage>, usize)> = STAGES
        .iter()
        .map(|(log, then, _)| {
            let state = State::fold(
                &then.iter().fold(Step::replayed(log), |script, step| script.then(step.actor, step.event())).records,
            );
            (state.project_stage(PROJECT.parse().unwrap()), state.anomalies().len())
        })
        .collect();
    let expected: Vec<(Option<ProjectStage>, usize)> = STAGES.iter().map(|(.., stage)| (Some(*stage), 0)).collect();
    assert_eq!(folded, expected);
}
