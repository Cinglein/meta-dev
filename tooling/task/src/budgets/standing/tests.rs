use meta_schema::Record;
use meta_severity::SeverityFilter;
use meta_state::State;
use settings::EmbeddedDefaults;

use super::{ChangeLink, Exposure, Standing};
use crate::budgets::budget::ItemFilter;
use crate::policy::Policy;
use crate::selection::Selection;

const LOG: &str = include_str!("tests/fixes.jsonl");
const TOP: &str = "features-and-maintenance";
const COST: &str = "performance-and-cost";
const CASES: [(&str, &str, &str, Exposure); 8] = [
    ("7", TOP, "minor", Exposure::PaysDown),
    ("7", TOP, "critical", Exposure::PaysDown),
    ("8", TOP, "minor", Exposure::PaysDown),
    ("8", TOP, "critical", Exposure::Within),
    ("7", COST, "critical", Exposure::Outside),
    ("8", COST, "minor", Exposure::Outside),
    ("9", TOP, "critical", Exposure::Outside),
    ("9", TOP, "minor", Exposure::Outside),
];

#[test]
fn a_cap_reaches_only_its_own_divisions_tasks_and_a_fix_pays_down_its_level_or_a_less_severe_one() {
    let policy = Policy::defaults();
    let records: Vec<Record> = LOG
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let state = State::fold(&records);
    let exposed = CASES.map(|(pull_request, division, level, _)| {
        let link = ChangeLink::new(pull_request.parse().ok(), None);
        let filter = ItemFilter::Severity(SeverityFilter::AtLevel(level.parse().unwrap()));
        let exposure = Standing::read(&state, &link, policy.scales())
            .exposure(Some(&filter), &division.parse().unwrap());
        (exposure, exposure.exposed(&Selection::Workspace).is_some())
    });
    let blocked = |exposure: Exposure| exposure == Exposure::Within;
    assert_eq!(
        exposed,
        CASES.map(|(.., exposure)| (exposure, blocked(exposure)))
    );
}
