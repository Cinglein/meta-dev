use meta_golden::Golden;

use super::{Brief, BriefOptions, Rules};

const TASK: &str = r#"{"task":"task-0123456789abcdef","text":"Print a brief","division":"features-and-maintenance"}"#;
const RULES: &str = "version: 7\n- first rule\n- second rule\n";
const ACTOR: &str = "claude-fixture";
const SCRATCH: &str = "/scratch";
const REPOSITORY: &str = "Cinglein/fixture";

#[test]
fn a_brief_gives_the_task_the_setup_and_the_rules_verbatim() {
    let options =
        BriefOptions::new(ACTOR.parse().unwrap(), Some(SCRATCH.parse().unwrap()), REPOSITORY.parse().unwrap());
    let brief = Brief::new(serde_json::from_str(TASK).unwrap(), RULES.parse::<Rules>().unwrap(), options);
    Golden::for_test(module_path!(), "brief").shown(&brief).unwrap();
}
