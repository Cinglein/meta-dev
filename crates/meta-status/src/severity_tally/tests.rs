use meta_schema::{Event, NewConcern, NewIssue, Record};
use meta_severity::Scales;
use meta_state::State;

use super::SeverityTally;

const TOP: &str = "features-and-maintenance";
const NAV: &str = "features-and-maintenance/nav";
const ROOT_CONCERN: &str = r#"{"kind":"ConcernIdentified","concern":"concern-0000000000000009","text":"x"}"#;

#[test]
fn the_tally_counts_each_level_up_the_tree_and_lists_each_item_under_its_own_division() {
    let rated =
        NewConcern::new(NAV.parse().unwrap(), TOP.parse().unwrap(), None, vec![]).rated("major".parse().unwrap());
    let unrated = NewIssue::new(TOP.parse().unwrap(), TOP.parse().unwrap());
    let shown = format!(
        "severity, open concerns and issues (counts roll up):\n{TOP}: critical 0, major 1, minor 0, unrated 1\n  unrated: {}\n{NAV}: critical 0, major 1, minor 0, unrated 0\n  major: {}\nroot:\n  unrated: concern-0000000000000009\n",
        unrated.issue(),
        rated.concern()
    );
    let events =
        [Event::ConcernIdentified(rated), Event::IssueReported(unrated), serde_json::from_str(ROOT_CONCERN).unwrap()];
    let records: Vec<Record> = events.into_iter().map(|event| Record::new(TOP.parse().unwrap(), event)).collect();
    let state = State::fold(&records);
    assert_eq!(SeverityTally::new(&state, &Scales::default()).to_string(), shown);
}
