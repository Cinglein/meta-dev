use meta_schema::{Division, Rating, Scenario, Text, TopDivision};

use crate::Scales;

const CHANGES: [(Option<&str>, &str, Option<&str>, &str); 7] = [
    (None, "minor", None, "minor"),
    (Some("major"), "major", None, "the item is already major"),
    (Some("major"), "minor", None, "minor (lowered from major)"),
    (Some("minor"), "critical", None, "raising from minor needs --scenario <reproduction or failure scenario>"),
    (Some("minor"), "critical", Some("crash on start"), "critical (raised from minor: crash on start)"),
    (None, "bogus", None, "bogus is not a level of the division's scale"),
    (Some("gone"), "minor", None, "raising from gone needs --scenario <reproduction or failure scenario>"),
];

#[test]
fn a_first_rating_and_a_lowering_are_free_and_a_raise_needs_a_scenario() {
    let scales = Scales::default();
    let scale = scales.scale(&Division::from(TopDivision::FeaturesAndMaintenance)).unwrap();
    let changed = CHANGES.map(|(current, level, scenario, _)| {
        let current = Rating::from(current.map(|level| level.parse().unwrap()));
        let scenario = scenario.map(|text| Scenario::from(text.parse::<Text>().unwrap()));
        let change = scale.change(&current, level.parse().unwrap(), scenario);
        change.map_or_else(|failure| failure.to_string(), |change| change.to_string())
    });
    assert_eq!(changed, CHANGES.map(|(.., expected)| expected.to_owned()));
}
