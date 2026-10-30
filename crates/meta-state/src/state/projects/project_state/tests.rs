use meta_schema::{Acceptance, NewProject};

use super::ProjectState;
use crate::state::projects::ProjectProgress;

const PROJECT: &str = r#"{"project":"project-00000000000000a1","text":"p","division":"architecture-and-planning"}"#;
const ACCEPTANCE: &str = r#"{"project":"project-00000000000000a1","root":"state","rows":[{"budget":"b","measure":"m","target":{"count":0}}],"spans":["fs"],"enforcement":[{"step":"test"}]}"#;
const NAMED: &str = "project-00000000000000a1 p @architecture-and-planning";

#[test]
fn display_names_the_project_then_its_stage() {
    let acceptance: Acceptance = serde_json::from_str(ACCEPTANCE).unwrap();
    let progress = [
        ProjectProgress::Proposed,
        ProjectProgress::Accepted(acceptance.clone()),
        ProjectProgress::Instrumented(acceptance.clone()),
        ProjectProgress::Delivered(acceptance.clone()),
        ProjectProgress::Regressed(acceptance),
    ];
    let shown: Vec<String> = progress
        .into_iter()
        .map(|progress| {
            let mut state = ProjectState::proposed(serde_json::from_str::<NewProject>(PROJECT).unwrap());
            state.advance(|_| Ok(progress)).unwrap();
            state.to_string()
        })
        .collect();
    let expected =
        ["proposed", "accepted", "instrumented", "delivered", "regressed"].map(|stage| format!("{NAMED} {stage}"));
    assert_eq!(shown, expected);
}
