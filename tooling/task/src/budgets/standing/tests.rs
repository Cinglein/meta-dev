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
const PLANNING: &str = "architecture-and-planning";
const CASES: [(&str, &str, Exposure); 4] = [
    ("7", "minor", Exposure::PaysDown),
    ("7", "critical", Exposure::PaysDown),
    ("8", "minor", Exposure::PaysDown),
    ("8", "critical", Exposure::Within),
];
const ACROSS: [(&str, &str, Option<&str>, bool); 14] = [
    ("9", TOP, Some("minor"), true),
    ("9", TOP, None, true),
    ("9", TOP, Some("major"), true),
    ("9", TOP, Some("critical"), false),
    ("10", TOP, Some("minor"), true),
    ("10", TOP, None, true),
    ("10", PLANNING, Some("minor"), true),
    ("10", COST, Some("minor"), true),
    ("10", TOP, Some("major"), false),
    ("10", TOP, Some("critical"), false),
    ("11", TOP, Some("minor"), false),
    ("11", TOP, None, true),
    ("11", TOP, Some("major"), false),
    ("7", COST, Some("critical"), false),
];

impl Standing {
    fn fixed() -> State {
        let records: Vec<Record> = LOG
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        State::fold(&records)
    }
}

#[test]
fn a_more_severe_fix_anywhere_passes_less_severe_caps_and_never_its_own_level_elsewhere() {
    let policy = Policy::defaults();
    let state = Standing::fixed();
    let passed = ACROSS.map(|(pull_request, division, level, _)| {
        let link = ChangeLink::new(pull_request.parse().ok(), None);
        let filter = ItemFilter::Severity(level.map_or(SeverityFilter::Unrated, |level| {
            SeverityFilter::AtLevel(level.parse().unwrap())
        }));
        Standing::read(&state, &link, policy.scales())
            .exposure(&filter, &division.parse().unwrap(), policy.domains())
            .exposed(&Selection::Workspace)
            .is_none()
    });
    assert_eq!(passed, ACROSS.map(|(.., passes)| passes));
}

#[test]
fn a_fix_pays_down_a_cap_at_its_item_level_or_a_less_severe_one_and_never_a_more_severe_one() {
    let policy = Policy::defaults();
    let state = Standing::fixed();
    let exposed = CASES.map(|(pull_request, level, _)| {
        let link = ChangeLink::new(pull_request.parse().ok(), None);
        let filter = ItemFilter::Severity(SeverityFilter::AtLevel(level.parse().unwrap()));
        Standing::read(&state, &link, policy.scales()).exposure(
            &filter,
            &TOP.parse().unwrap(),
            policy.domains(),
        )
    });
    assert_eq!(exposed, CASES.map(|(.., exposure)| exposure));
}
