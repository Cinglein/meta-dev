use std::collections::BTreeMap;

use meta_schema::{Division, Event, NewConcern, Record, Target};
use meta_state::State;

use crate::{Scale, Scales};

const TOP: &str = "features-and-maintenance";
const PAY: &str = "features-and-maintenance/pay";
const OFFLINE: &str = "features-and-maintenance/nav/offline";
const PAY_BEFORE: &str = r#"{"features-and-maintenance/pay": [
    {"name": "lost", "meaning": "x", "parent": "critical"},
    {"name": "slow", "meaning": "x", "parent": "major"},
    {"name": "typo", "meaning": "x", "parent": "minor"}
]}"#;
const PAY_REMAPPED: &str = r#"{"features-and-maintenance/pay": [
    {"name": "lost", "meaning": "x", "parent": "critical"},
    {"name": "fast", "meaning": "x", "parent": "major"},
    {"name": "slow", "meaning": "x", "parent": "minor"}
]}"#;
const REORDERED: &str = r#"{"features-and-maintenance": [
    {"name": "major", "meaning": "x"}, {"name": "critical", "meaning": "x"}, {"name": "minor", "meaning": "x"}
]}"#;
const NAV_BEFORE: &str = r#"{"features-and-maintenance/nav": [
    {"name": "lost", "meaning": "x", "parent": "critical"},
    {"name": "wrong-turn", "meaning": "x", "parent": "critical"},
    {"name": "late", "meaning": "x", "parent": "major"},
    {"name": "typo", "meaning": "x", "parent": "minor"}
], "features-and-maintenance/nav/offline": [
    {"name": "x", "meaning": "x", "parent": "lost"},
    {"name": "w", "meaning": "x", "parent": "wrong-turn"},
    {"name": "s", "meaning": "x", "parent": "late"},
    {"name": "t", "meaning": "x", "parent": "typo"}
]}"#;
const NAV_SWAPPED: &str = r#"{"features-and-maintenance/nav": [
    {"name": "lost", "meaning": "x", "parent": "critical"},
    {"name": "wrong-turn", "meaning": "x", "parent": "critical"},
    {"name": "late", "meaning": "x", "parent": "major"},
    {"name": "typo", "meaning": "x", "parent": "minor"}
], "features-and-maintenance/nav/offline": [
    {"name": "w", "meaning": "x", "parent": "lost"},
    {"name": "x", "meaning": "x", "parent": "wrong-turn"},
    {"name": "s", "meaning": "x", "parent": "late"},
    {"name": "t", "meaning": "x", "parent": "typo"}
]}"#;
const FINER: &str = r#"{"features-and-maintenance/nav": [
    {"name": "lost", "meaning": "x", "parent": "critical"},
    {"name": "late", "meaning": "x", "parent": "major"},
    {"name": "typo", "meaning": "x", "parent": "minor"}
]}"#;
const DEFAULTS: &str = "{}";
const CASES: [(&str, &str, &str, &str, &str); 5] = [
    (
        PAY,
        "slow",
        PAY_BEFORE,
        PAY_REMAPPED,
        "lowered by policy: ITEM at features-and-maintenance: major became minor; lower it with meta rate\n",
    ),
    (
        TOP,
        "critical",
        DEFAULTS,
        REORDERED,
        "not comparable: ITEM at features-and-maintenance: the scale's levels changed\n",
    ),
    (
        OFFLINE,
        "x",
        NAV_BEFORE,
        NAV_SWAPPED,
        "lowered by policy: ITEM at features-and-maintenance/nav: lost became wrong-turn; lower it with meta rate\n",
    ),
    (OFFLINE, "x", NAV_SWAPPED, NAV_BEFORE, ""),
    ("features-and-maintenance/nav", "critical", DEFAULTS, FINER, ""),
];
const ITEM: &str = "ITEM";

struct Case;

impl Case {
    fn scales(json: &str) -> Scales {
        Scales::try_from(serde_json::from_str::<BTreeMap<Division, Scale>>(json).unwrap()).unwrap()
    }

    fn shown(division: &str, level: &str, before: &str, now: &str) -> String {
        let concern = NewConcern::new(division.parse().unwrap(), TOP.parse().unwrap(), None, vec![])
            .rated(level.parse().unwrap());
        let item = Target::Concern(concern.concern()).to_string();
        let state = State::fold(&[Record::new(TOP.parse().unwrap(), Event::ConcernIdentified(concern))]);
        Self::scales(now).demotions_from(&Self::scales(before), &state).to_string().replace(&item, ITEM)
    }
}

#[test]
fn a_policy_change_may_raise_or_refine_an_open_item_but_never_lower_it_or_reorder_its_scale() {
    let shown = CASES.map(|(division, level, before, now, _)| Case::shown(division, level, before, now));
    assert_eq!(shown, CASES.map(|(.., expected)| expected.to_owned()));
}
