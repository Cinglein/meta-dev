use std::collections::BTreeMap;

use meta_schema::{Division, LevelName, TopDivision};

use crate::{Scale, Scales};

const NESTED: &str = r#"{
  "features-and-maintenance/nav": [
    {"name": "lost", "meaning": "x", "parent": "critical"},
    {"name": "wrong-turn", "meaning": "x", "parent": "critical"},
    {"name": "slow", "meaning": "x", "parent": "major"},
    {"name": "typo", "meaning": "x", "parent": "minor"}
  ],
  "features-and-maintenance/nav/offline": [
    {"name": "no-map", "meaning": "x", "parent": "lost"},
    {"name": "no-turn", "meaning": "x", "parent": "wrong-turn"},
    {"name": "stale", "meaning": "x", "parent": "slow"},
    {"name": "blurry", "meaning": "x", "parent": "typo"}
  ]
}"#;
const TOP: &str = "features-and-maintenance";
const NAV: &str = "features-and-maintenance/nav";
const BELOW_NAV: &str = "features-and-maintenance/nav/x";
const OFFLINE: &str = "features-and-maintenance/nav/offline";
const DEEP_DEFAULT: &str = "performance-and-cost/ci/tests";
const DEFAULT_LEVELS: [&str; 3] = ["critical", "major", "minor"];
const ROLLED: [(&str, &str, &str, Option<&str>); 6] = [
    (OFFLINE, "stale", TOP, Some("major")),
    (OFFLINE, "stale", NAV, Some("slow")),
    (OFFLINE, "no-map", OFFLINE, Some("no-map")),
    (OFFLINE, "no-map", TOP, Some("critical")),
    ("features-and-maintenance/nav/offline/x", "stale", BELOW_NAV, Some("slow")),
    (TOP, "critical", NAV, None),
];
const REFUSED: [(&str, &str); 8] = [
    (
        r#"{"features-and-maintenance/pay": [{"name": "outage", "meaning": "x", "parent": "critical"}, {"name": "degraded", "meaning": "x", "parent": "major"}, {"name": "critical", "meaning": "x", "parent": "minor"}]}"#,
        "scale features-and-maintenance/pay: critical names a parent level but maps onto another",
    ),
    (
        r#"{"features-and-maintenance/a": [{"name": "b", "meaning": "x", "parent": "minor"}]}"#,
        "scale features-and-maintenance/a: no level maps onto critical",
    ),
    (
        r#"{"features-and-maintenance/a": [{"name": "b", "meaning": "x", "parent": "minor"}, {"name": "c", "meaning": "x", "parent": "major"}]}"#,
        "scale features-and-maintenance/a: c maps above a more severe level",
    ),
    (
        r#"{"features-and-maintenance/a": [{"name": "b", "meaning": "x", "parent": "nope"}]}"#,
        "scale features-and-maintenance/a: b maps onto no level of the parent's scale",
    ),
    (
        r#"{"features-and-maintenance/a": [{"name": "b", "meaning": "x"}]}"#,
        "scale features-and-maintenance/a: b maps onto no level of the parent's scale",
    ),
    (
        r#"{"performance-and-cost": [{"name": "b", "meaning": "x", "parent": "major"}]}"#,
        "scale performance-and-cost: b maps onto a parent, but a top division has none",
    ),
    (
        r#"{"performance-and-cost": [{"name": "b", "meaning": "x"}, {"name": "b", "meaning": "y"}]}"#,
        "scale performance-and-cost: b twice",
    ),
    (r#"{"performance-and-cost/a": []}"#, "scale performance-and-cost/a: no levels"),
];

impl Scales {
    fn declared(json: &str) -> Option<Scales> {
        Scales::try_from(serde_json::from_str::<BTreeMap<Division, Scale>>(json).ok()?).ok()
    }

    fn refusal(json: &str) -> Option<String> {
        Scales::try_from(serde_json::from_str::<BTreeMap<Division, Scale>>(json).ok()?)
            .err()
            .map(|failure| failure.to_string())
    }
}

impl Scale {
    fn names(&self) -> Vec<String> {
        self.levels().iter().map(|level| level.name().to_string()).collect()
    }
}

#[test]
fn a_division_without_a_scale_takes_its_nearest_declared_ancestors() {
    let nested = Scales::declared(NESTED).unwrap();
    let defaults = Scales::default();
    let deep: Division = DEEP_DEFAULT.parse().unwrap();
    assert_eq!(defaults.owner(&deep), Some(&Division::from(TopDivision::PerformanceAndCost)));
    assert_eq!(defaults.scale(&deep).map(Scale::names), Some(DEFAULT_LEVELS.map(str::to_owned).to_vec()));
    assert_eq!(nested.owner(&BELOW_NAV.parse().unwrap()).map(ToString::to_string), Some(NAV.to_owned()));
}

#[test]
fn a_child_level_rolls_up_through_each_mapping_and_only_onto_divisions_above_it() {
    let nested = Scales::declared(NESTED).unwrap();
    let rolled = ROLLED.map(|(from, level, onto, _)| {
        let level: LevelName = level.parse().unwrap();
        nested.rolled_up(&from.parse().unwrap(), &level, &onto.parse().unwrap()).map(|level| level.to_string())
    });
    assert_eq!(rolled, ROLLED.map(|(.., expected)| expected.map(str::to_owned)));
    assert_eq!(serde_json::from_str::<Scales>(&serde_json::to_string(&nested).unwrap()).unwrap(), nested);
}

#[test]
fn order_breaking_unmapped_top_mapped_repeated_and_empty_scales_are_refused() {
    let refused = REFUSED.map(|(json, _)| Scales::refusal(json));
    assert_eq!(refused, REFUSED.map(|(_, failure)| Some(failure.to_owned())));
}
