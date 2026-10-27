use meta_schema::Record;
use meta_severity::SeverityFilter;
use meta_state::State;
use settings::EmbeddedDefaults;

use super::{ChangeLink, Exposure, Standing};
use crate::budgets::budget::ItemFilter;
use crate::policy::Policy;

const LOG: &str = include_str!("tests/fixes.jsonl");
const TOP: &str = "features-and-maintenance";
const CASES: [(&str, &str, Exposure); 4] = [
    ("7", "minor", Exposure::PaysDown),
    ("7", "critical", Exposure::PaysDown),
    ("8", "minor", Exposure::PaysDown),
    ("8", "critical", Exposure::Within),
];

#[test]
fn a_fix_pays_down_a_cap_at_its_item_level_or_a_less_severe_one_and_never_a_more_severe_one() {
    let policy = Policy::defaults();
    let records: Vec<Record> = LOG
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let state = State::fold(&records);
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
