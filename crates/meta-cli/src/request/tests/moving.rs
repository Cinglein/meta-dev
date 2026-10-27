use meta_log::LogStore;
use meta_schema::Record;
use meta_severity::Scales;
use meta_state::State;
use meta_status::SeverityTally;

use super::{DIVISION, ID_INDEX, Meta, OWNER};
use crate::request::cli_failure::CliFailure;

const ROOT_ISSUE: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"IssueReported","issue":"issue-00000000000000aa","text":"crash"}"#;
const ISSUE: &str = "issue-00000000000000aa";
const COST: &str = "performance-and-cost";
const NAV: &str = "features-and-maintenance/nav";
const A: &str = "features-and-maintenance/a";
const B: &str = "features-and-maintenance/b";
const NOWHERE: &str = "features-and-maintenance/nowhere";
const SCALES: &str = r#"{
    "features-and-maintenance/nav": [{"name": "lost", "meaning": "x", "parent": "critical"}, {"name": "slow", "meaning": "x", "parent": "major"}, {"name": "typo", "meaning": "x", "parent": "minor"}],
    "features-and-maintenance/a": [{"name": "x", "meaning": "x", "parent": "critical"}, {"name": "y", "meaning": "x", "parent": "major"}, {"name": "z", "meaning": "x", "parent": "minor"}],
    "features-and-maintenance/b": [{"name": "y", "meaning": "x", "parent": "critical"}, {"name": "x", "meaning": "x", "parent": "major"}, {"name": "w", "meaning": "x", "parent": "minor"}]
}"#;
const MOVE: &str = "move";
const TO: &str = "--division";
const SEVERITY: &str = "--severity";
const RATED_PLACED: &str = "crash @features-and-maintenance !major";
const KEPT: &str = "crash @performance-and-cost !major";
const COUNTED: &str =
    "performance-and-cost: critical 0, major 1, minor 0, unrated 0\n  major: issue-00000000000000aa\n";
const REFUSED: [&str; 6] = [
    "x means another level on features-and-maintenance/b; move it with --severity",
    "y means another level on features-and-maintenance/b; move it with --severity",
    "major is not on the scale of features-and-maintenance/nav; move it with --severity",
    "raising from z needs --scenario",
    "raising from major needs --scenario",
    "--scenario needs --severity",
];
const MOVED: [&str; 4] =
    ["!y (same as x)", "!w (lowered from y)", "!slow (same as major)", "!minor (lowered from slow)"];

impl Meta {
    fn seeded(line: &str) -> Meta {
        let meta = Meta::default();
        let record: Record = serde_json::from_str(line).unwrap();
        meta.0.append::<CliFailure>(&mut |_| Ok(record.clone())).unwrap();
        meta
    }
}

#[test]
fn a_root_item_moves_into_a_division_then_rates_and_lists_counts_and_moves_with_it() {
    let meta = Meta::seeded(ROOT_ISSUE);
    assert!(meta.run(OWNER, &["rate", ISSUE, "major"]).unwrap_err().to_string().contains("no scale"));
    assert!(meta.run(OWNER, &[MOVE, ISSUE, TO, DIVISION]).unwrap().contains(&format!("ItemMoved {ISSUE}")));
    meta.run(OWNER, &["rate", ISSUE, "major"]).unwrap();
    assert!(meta.run(OWNER, &["list", "issues"]).unwrap().contains(RATED_PLACED));
    assert!(meta.run(OWNER, &[MOVE, ISSUE, TO, COST]).unwrap().trim_end().ends_with("to @performance-and-cost !major"));
    let written = meta.0.read().unwrap().records().len();
    let refusals = [[MOVE, ISSUE, TO, COST], [MOVE, ISSUE, TO, NOWHERE], [MOVE, ISSUE, "--severity", "major"]];
    assert!(refusals.iter().all(|words| meta.run(OWNER, words).is_err()));
    assert!(meta.run(OWNER, &[MOVE, ISSUE, TO, NOWHERE]).unwrap_err().to_string().contains(NOWHERE));
    assert!(meta.run(OWNER, &["list", "issues"]).unwrap().contains(KEPT));
    let tally = SeverityTally::new(&State::fold(meta.0.read().unwrap().records()), &Scales::default()).to_string();
    assert!(tally.contains(COUNTED) && !tally.contains("root:") && meta.0.read().unwrap().records().len() == written);
}

#[test]
fn a_move_compares_levels_where_they_roll_up_so_only_an_equal_roll_up_is_kept_and_a_raise_needs_a_scenario() {
    let meta = Meta::default();
    let scales: Scales = serde_json::from_str(SCALES).unwrap();
    let concern = |division: &str, level: &str| {
        let written = meta.under(&scales, &["concern", "c", TO, division, SEVERITY, level]).unwrap();
        written.split_whitespace().nth(ID_INDEX).unwrap().to_owned()
    };
    let ids = [(A, "x"), (A, "y"), (A, "z"), (DIVISION, "major")].map(|(at, level)| concern(at, level));
    let [ax, ay, az, top] = ids.each_ref().map(String::as_str);
    let moved = |words: &[&str]| meta.under(&scales, &[&[MOVE][..], words].concat()).map_err(|error| error.to_string());
    let refused = [
        moved(&[ax, TO, B]),
        moved(&[ay, TO, B]),
        moved(&[top, TO, NAV]),
        moved(&[az, TO, B, SEVERITY, "x"]),
        moved(&[top, TO, NAV, SEVERITY, "lost"]),
        moved(&[top, TO, NAV, "--scenario", "s"]),
    ];
    assert!(refused.iter().zip(REFUSED).all(|(refusal, why)| refusal.as_ref().is_err_and(|shown| shown.contains(why))));
    let done = [
        moved(&[ax, TO, B, SEVERITY, "y"]),
        moved(&[ay, TO, B, SEVERITY, "w"]),
        moved(&[top, TO, NAV, SEVERITY, "slow"]),
        moved(&[top, TO, DIVISION, SEVERITY, "minor"]),
    ];
    assert!(done.iter().zip(MOVED).all(|(written, shown)| written.as_ref().is_ok_and(|line| line.contains(shown))));
    assert!(meta.run(OWNER, &["list", "concerns"]).unwrap().contains(&format!("{top} c @{DIVISION} !minor")));
    meta.run(OWNER, &["retire", az, "dropped"]).unwrap();
    assert!(moved(&[az, TO, B, SEVERITY, "w"]).is_err_and(|refusal| refusal.contains("retired")));
}
