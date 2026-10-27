use std::collections::BTreeMap;

use crate::{Division, LevelName, Scale, Scales, TopDivision};

const NESTED: &str = r#"{
  "features-and-maintenance/nav": [
    {"name": "lost", "meaning": "x", "parent": "critical"},
    {"name": "wrong-turn", "meaning": "x", "parent": "critical"},
    {"name": "slow", "meaning": "x", "parent": "major"}
  ],
  "features-and-maintenance/nav/offline": [
    {"name": "no-map", "meaning": "x", "parent": "lost"},
    {"name": "stale", "meaning": "x", "parent": "slow"}
  ]
}"#;
const TOP: &str = "features-and-maintenance";
const NAV: &str = "features-and-maintenance/nav";
const OFFLINE: &str = "features-and-maintenance/nav/offline";
const DEEP_DEFAULT: &str = "performance-and-cost/ci/tests";
const DEFAULT_LEVELS: [&str; 3] = ["critical", "major", "minor"];
const ROLLED: [(&str, &str, &str, Option<&str>); 6] = [
    (OFFLINE, "stale", TOP, Some("major")),
    (OFFLINE, "stale", NAV, Some("slow")),
    (OFFLINE, "no-map", OFFLINE, Some("no-map")),
    (OFFLINE, "no-map", TOP, Some("critical")),
    ("features-and-maintenance/nav/offline/x", "stale", "features-and-maintenance/nav/y", Some("slow")),
    (TOP, "critical", NAV, None),
];
const REFUSED: [(&str, &str); 6] = [
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

fn names(scale: &Scale) -> Vec<String> {
    scale.levels().iter().map(|level| level.name().to_string()).collect()
}

fn scales(json: &str) -> Result<Scales, String> {
    let declared: BTreeMap<Division, Scale> = serde_json::from_str(json).map_err(|error| error.to_string())?;
    Scales::try_from(declared).map_err(|failure| failure.to_string())
}

#[test]
fn a_division_without_a_scale_takes_its_nearest_declared_ancestors() {
    let nested = scales(NESTED).unwrap();
    let defaults = Scales::default();
    let deep = defaults.effective(&DEEP_DEFAULT.parse().unwrap()).map(|(owner, scale)| (owner.clone(), names(scale)));
    let top = Division::from(TopDivision::PerformanceAndCost);
    assert_eq!(deep, Some((top, DEFAULT_LEVELS.map(str::to_owned).to_vec())));
    let below = format!("{NAV}/x").parse().unwrap();
    assert_eq!(nested.effective(&below).map(|(owner, _)| owner.to_string()), Some(NAV.to_owned()));
}

#[test]
fn a_child_level_rolls_up_through_each_mapping_and_only_onto_divisions_above_it() {
    let nested = scales(NESTED).unwrap();
    let rolled = ROLLED.map(|(from, level, onto, _)| {
        let level: LevelName = level.parse().unwrap();
        nested.rolled_up(&from.parse().unwrap(), &level, &onto.parse().unwrap()).map(|level| level.to_string())
    });
    assert_eq!(rolled, ROLLED.map(|(.., expected)| expected.map(str::to_owned)));
}

#[test]
fn order_breaking_unmapped_top_mapped_repeated_and_empty_scales_are_refused() {
    let refused = REFUSED.map(|(json, _)| scales(json).err());
    assert_eq!(refused, REFUSED.map(|(_, failure)| Some(failure.to_owned())));
}
