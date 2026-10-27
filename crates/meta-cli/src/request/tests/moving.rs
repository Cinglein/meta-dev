use meta_log::LogStore;
use meta_schema::Record;
use meta_severity::Scales;
use meta_state::State;
use meta_status::SeverityTally;

use super::{DIVISION, Meta, OWNER};
use crate::request::cli_failure::CliFailure;

const ROOT_ISSUE: &str = r#"{"at":"2026-10-25T00:00:00Z","actor":"a","kind":"IssueReported","issue":"issue-00000000000000aa","text":"crash"}"#;
const ISSUE: &str = "issue-00000000000000aa";
const COST: &str = "performance-and-cost";
const NAV: &str = "features-and-maintenance/nav";
const NOWHERE: &str = "features-and-maintenance/nowhere";
const NAV_SCALE: &str = r#"{"features-and-maintenance/nav": [{"name": "lost", "meaning": "x", "parent": "critical"}, {"name": "slow", "meaning": "x", "parent": "major"}, {"name": "typo", "meaning": "x", "parent": "minor"}]}"#;
const MOVE: &str = "move";
const TO: &str = "--division";
const SEVERITY: &str = "--severity";
const NEEDS_SEVERITY: &str = "major is not on the scale of features-and-maintenance/nav; move it with --severity";
const SCENARIO: [&str; 2] = ["--scenario", "offline start shows no map"];
const RATED_CONCERN: [&str; 6] = ["concern", "map", "lost", "--division", DIVISION, "--severity"];
const RATED_PLACED: &str = "crash @features-and-maintenance !major";
const KEPT: &str = "crash @performance-and-cost !major";
const COUNTED: &str =
    "performance-and-cost: critical 0, major 1, minor 0, unrated 0\n  major: issue-00000000000000aa\n";
const RAISED: &str = "!slow (raised from major: offline start shows no map)";
const LOWERED: &str = "!minor (lowered from major)";

impl Meta {
    fn seeded(line: &str) -> Meta {
        let meta = Meta::default();
        let record: Record = serde_json::from_str(line).unwrap();
        meta.0.append::<CliFailure>(&mut |_| Ok(record.clone())).unwrap();
        meta
    }

    fn refusal(&self, scales: &Scales, words: &[&str]) -> String {
        self.under(scales, words).unwrap_err().to_string()
    }
}

#[test]
fn a_root_item_moves_into_a_division_then_rates_and_lists_counts_and_moves_with_it() {
    let meta = Meta::seeded(ROOT_ISSUE);
    assert!(meta.run(OWNER, &["rate", ISSUE, "major"]).unwrap_err().to_string().contains("no scale"));
    assert!(meta.run(OWNER, &[MOVE, ISSUE, TO, DIVISION]).unwrap().contains(&format!("ItemMoved {ISSUE}")));
    meta.run(OWNER, &["rate", ISSUE, "major"]).unwrap();
    assert!(meta.run(OWNER, &["list", "issues"]).unwrap().contains(RATED_PLACED));
    meta.run(OWNER, &[MOVE, ISSUE, TO, COST]).unwrap();
    let written = meta.0.read().unwrap().records().len();
    let refusals = [[MOVE, ISSUE, TO, COST], [MOVE, ISSUE, TO, NOWHERE], [MOVE, ISSUE, "--severity", "major"]];
    assert!(refusals.iter().all(|words| meta.run(OWNER, words).is_err()));
    assert!(meta.run(OWNER, &[MOVE, ISSUE, TO, NOWHERE]).unwrap_err().to_string().contains(NOWHERE));
    assert!(meta.run(OWNER, &["list", "issues"]).unwrap().contains(KEPT));
    let tally = SeverityTally::new(&State::fold(meta.0.read().unwrap().records()), &Scales::default()).to_string();
    assert!(tally.contains(COUNTED) && !tally.contains("root:") && meta.0.read().unwrap().records().len() == written);
}

#[test]
fn a_rated_item_keeps_its_level_only_where_the_scale_has_it_and_otherwise_takes_a_severity_by_the_rating_rules() {
    let meta = Meta::default();
    let nav: Scales = serde_json::from_str(NAV_SCALE).unwrap();
    let concern = meta.written(OWNER, &[&RATED_CONCERN[..], &["major"]].concat());
    let other = meta.written(OWNER, &[&RATED_CONCERN[..], &["major"]].concat());
    assert!(meta.refusal(&nav, &[MOVE, &concern, TO, NAV]).contains(NEEDS_SEVERITY));
    assert!(meta.refusal(&nav, &[MOVE, &concern, TO, NAV, SEVERITY, "slow"]).contains("needs --scenario"));
    assert!(meta.refusal(&nav, &[MOVE, &concern, TO, NAV, SEVERITY, "blocker"]).contains("not a level"));
    let raised = meta.under(&nav, &[&[MOVE, concern.as_str(), TO, NAV, SEVERITY, "slow"][..], &SCENARIO[..]].concat());
    assert!(raised.unwrap().contains(RAISED));
    assert!(meta.run(OWNER, &["list", "concerns"]).unwrap().contains(&format!("{concern} map lost @{NAV} !slow")));
    assert!(meta.refusal(&nav, &[MOVE, &other, TO, COST, SEVERITY, "critical"]).contains("needs --scenario"));
    assert!(meta.under(&nav, &[MOVE, &other, TO, COST, SEVERITY, "minor"]).unwrap().contains(LOWERED));
    meta.run(OWNER, &["retire", &other, "dropped"]).unwrap();
    assert!(meta.refusal(&nav, &[MOVE, &other, TO, DIVISION]).contains("retired"));
}
