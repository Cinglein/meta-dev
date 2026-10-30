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
const CONCERN: [&str; 4] = ["concern", "slow", "--division", DIVISION];
const OPEN_WORK: [&str; 4] = ["open", "work", "--division", DIVISION];

impl Meta {
    fn plant(&self, jsons: &[&str]) {
        let append = |json: &&str| self.0.append::<CliFailure>(&mut |_| Ok(serde_json::from_str(json).unwrap()));
        jsons.iter().try_for_each(|json| append(json).map(drop)).unwrap();
    }

    fn refused(&self, actor: &str, words: &[&str]) -> String {
        self.run(actor, words).unwrap_err().to_string()
    }

    fn opened(&self, purpose: &[&str]) -> String {
        self.written(ALICE, &[&OPEN_WORK[..], purpose].concat())
    }

    fn accepted_project(&self) -> String {
        let project = self.written(ALICE, &PROPOSE);
        self.run(OWNER, &[&["project", "accept", &project][..], &ACCEPT_WITH[..]].concat()).unwrap();
        project
    }
}

#[test]
fn a_project_is_proposed_accepted_listed_and_only_then_served() {
    let meta = Meta::default();
    let project = meta.written(ALICE, &PROPOSE);
    assert!(meta.run(ALICE, &LIST).unwrap().contains(&project));
    assert!(meta.run(ALICE, &LIST).unwrap().contains("proposed"));
    assert!(meta.refused(ALICE, &[&OPEN_WORK[..], &["--serves", &project]].concat()).contains("proposed"));
    let accept = |actor: &str| meta.run(actor, &[&["project", "accept", &project][..], &ACCEPT_WITH[..]].concat());
    assert!(accept(ALICE).unwrap_err().to_string().contains("owner"));
    assert!(meta.refused(OWNER, &["project", "accept", &project, "--root", "state"]).contains("--row"));
    assert!(accept(OWNER).is_ok());
    assert!(meta.run(ALICE, &LIST).unwrap().contains("accepted"));
    assert!(meta.refused(ALICE, &["project", "list", "--all"]).contains("--all"));
    assert!(meta.refused(ALICE, &["list", "projects", "--all"]).contains("--all"));
    assert!(meta.run(ALICE, &["list", "projects"]).unwrap().contains(&project));
}

#[test]
fn tasks_open_with_one_purpose_and_serving_names_the_way_out() {
    let meta = Meta::default();
    let project = meta.accepted_project();
    let concern = meta.written(ALICE, &CONCERN);
    let serving = meta.opened(&["--serves", &project]);
    let instrumenting = meta.opened(&["--instruments", &project]);
    let caused = meta.opened(&["--caused-by", &concern]);
    let both = [&OPEN_WORK[..], &["--serves", &project, "--instruments", &project]].concat();
    assert!(meta.refused(ALICE, &both).contains("--instruments cannot be given after --serves"));
    let twice = [&OPEN_WORK[..], &["--caused-by", &concern, "--caused-by", &concern]].concat();
    assert!(meta.refused(ALICE, &twice).contains("cannot be given after"));
    let rooted = ["project", "accept", &project, "--root", "state", "--root", "process"];
    assert!(meta.refused(OWNER, &rooted).contains("--root cannot be given after --root"));
    assert!(
        meta.refused(ALICE, &[&OPEN_WORK[..], &["--serves", &project, "--project", &project]].concat())
            .contains("--caused-by")
    );
    let refusal = meta.refused(ALICE, &[CLAIM, &serving]);
    assert!(refusal.contains("accepted") && refusal.contains(&instrumenting), "{refusal}");
    assert!(meta.run(ALICE, &[CLAIM, &instrumenting]).is_ok() && meta.run(ALICE, &[CLAIM, &caused]).is_ok());
    let listed = meta.run(ALICE, &["list", "tasks"]).unwrap();
    assert!(listed.contains(&format!("serves {project}")) && listed.contains(&format!("instruments {project}")));
    assert!(listed.contains(&format!("caused_by {concern}")));
}

#[test]
fn a_cause_names_a_real_concern_issue_or_task_and_a_pull_is_refused_for_now() {
    let meta = Meta::default();
    let concern = meta.written(ALICE, &CONCERN);
    let pull = meta.refused(ALICE, &[&OPEN_WORK[..], &["--caused-by", PULL]].concat());
    assert!(pull.contains("pull causes need the merged-PR check; cite the concern or task"));
    let ghost = meta.refused(ALICE, &[&OPEN_WORK[..], &["--caused-by", OLD_ONE]].concat());
    assert!(ghost.contains("there is no entry"));
    let task = meta.opened(&["--caused-by", &concern]);
    assert!(meta.refused(ALICE, &["cause", &task, &task]).contains("the task itself"));
    assert!(meta.run(ALICE, &[&OPEN_WORK[..], &["--caused-by", &task]].concat()).is_ok());
}

#[test]
fn an_old_task_without_a_purpose_gets_one_once_and_is_claimable_with_or_without_it() {
    let meta = Meta::default();
    meta.plant(&PLANTED);
    let project = meta.accepted_project();
    let concern = meta.written(ALICE, &CONCERN);
    meta.run(ALICE, &["serve", OLD_ONE, &project]).unwrap();
    meta.run(ALICE, &["instrument", OLD_TWO, &project]).unwrap();
    assert!(meta.refused(ALICE, &["serve", OLD_TWO, &project]).contains("already has a purpose"));
    assert!(meta.refused(ALICE, &[CLAIM, OLD_ONE]).contains(OLD_TWO));
    assert!(meta.run(ALICE, &[CLAIM, OLD_TWO]).is_ok());
    assert!(meta.run(ALICE, &[CLAIM, OLD_THREE]).is_ok());
    meta.run(ALICE, &["cause", OLD_THREE, &concern, "--project", &project]).unwrap();
}
