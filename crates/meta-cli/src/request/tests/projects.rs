use meta_log::LogStore;

use super::{ALICE, CLAIM, DIVISION, Meta, OWNER};
use crate::request::cli_failure::CliFailure;

const PLANTED: [&str; 3] = [
    r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-00000000000000d1","text":"old one"}"#,
    r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-00000000000000d2","text":"old two"}"#,
    r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"TaskOpened","task":"task-00000000000000d3","text":"old three"}"#,
];
const OLD_ONE: &str = "task-00000000000000d1";
const OLD_TWO: &str = "task-00000000000000d2";
const OLD_THREE: &str = "task-00000000000000d3";
const PROPOSE: [&str; 5] = ["project", "propose", "faster", "--division", DIVISION];
const LIST: [&str; 2] = ["project", "list"];
const ROW: &str = "speed:test_time<=0";
const ACCEPT_WITH: [&str; 8] = ["--root", "state", "--row", ROW, "--span", "fs", "--gate", "step:test"];
const PULL: &str = "7";

impl Meta {
    fn plant(&self, json: &str) {
        self.0.append::<CliFailure>(&mut |_| Ok(serde_json::from_str(json).unwrap())).unwrap();
    }

    fn refused(&self, actor: &str, words: &[&str]) -> String {
        self.run(actor, words).unwrap_err().to_string()
    }

    fn opened(&self, purpose: &[&str]) -> String {
        self.written(ALICE, &[&["open", "work", "--division", DIVISION][..], purpose].concat())
    }
}

#[test]
fn a_project_is_proposed_accepted_listed_and_only_then_served() {
    let meta = Meta::default();
    let project = meta.written(ALICE, &PROPOSE);
    assert!(meta.run(ALICE, &LIST).unwrap().contains(&format!("{project}")));
    assert!(meta.run(ALICE, &LIST).unwrap().contains("proposed"));
    assert!(meta.refused(ALICE, &["open", "w", "--division", DIVISION, "--serves", &project]).contains("proposed"));
    let accept = |actor: &str, extra: &[&str]| {
        meta.run(actor, &[&["project", "accept", &project][..], extra, &ACCEPT_WITH[..]].concat())
    };
    assert!(accept(ALICE, &[]).unwrap_err().to_string().contains("owner"));
    assert!(meta.refused(OWNER, &["project", "accept", &project, "--root", "state"]).contains("--row"));
    assert!(accept(OWNER, &[]).is_ok());
    assert!(meta.run(ALICE, &LIST).unwrap().contains("accepted"));
}

#[test]
fn tasks_open_with_one_purpose_and_serving_waits_for_instrumentation() {
    let meta = Meta::default();
    let project = meta.written(ALICE, &PROPOSE);
    meta.run(OWNER, &[&["project", "accept", &project][..], &ACCEPT_WITH[..]].concat()).unwrap();
    let serving = meta.opened(&["--serves", &project]);
    let instrumenting = meta.opened(&["--instruments", &project]);
    let caused = meta.opened(&["--caused-by", PULL]);
    assert!(meta.refused(ALICE, &["open", "w", "--division", DIVISION]).contains("purpose"));
    let both = ["--serves", &project, "--instruments", &project];
    assert!(meta.refused(ALICE, &[&["open", "w", "--division", DIVISION][..], &both].concat()).contains("one"));
    assert!(meta.refused(ALICE, &[CLAIM, &serving]).contains("not instrumented"));
    assert!(meta.run(ALICE, &[CLAIM, &instrumenting]).is_ok() && meta.run(ALICE, &[CLAIM, &caused]).is_ok());
    let listed = meta.run(ALICE, &["list", "tasks"]).unwrap();
    assert!(listed.contains(&format!("serves {project}")) && listed.contains(&format!("instruments {project}")));
    assert!(listed.contains("caused_by #7"));
}

#[test]
fn an_unlinked_task_is_not_claimed_until_it_gets_a_purpose_once() {
    let meta = Meta::default();
    let project = meta.written(ALICE, &PROPOSE);
    meta.run(OWNER, &[&["project", "accept", &project][..], &ACCEPT_WITH[..]].concat()).unwrap();
    PLANTED.iter().for_each(|json| meta.plant(json));
    let refusal = meta.refused(ALICE, &[CLAIM, OLD_ONE]);
    assert!(["no purpose", "meta serve", "meta instrument", "meta cause"].iter().all(|word| refusal.contains(word)));
    meta.run(ALICE, &["serve", OLD_ONE, &project]).unwrap();
    meta.run(ALICE, &["instrument", OLD_TWO, &project]).unwrap();
    meta.run(ALICE, &["cause", OLD_THREE, PULL, "--project", &project]).unwrap();
    assert!(meta.refused(ALICE, &["serve", OLD_TWO, &project]).contains("already has a purpose"));
    assert!(meta.refused(ALICE, &[CLAIM, OLD_ONE]).contains("not instrumented"));
    assert!(meta.run(ALICE, &[CLAIM, OLD_TWO]).is_ok() && meta.run(ALICE, &[CLAIM, OLD_THREE]).is_ok());
}
