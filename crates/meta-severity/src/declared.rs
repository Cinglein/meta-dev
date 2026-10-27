use std::collections::BTreeMap;

use meta_schema::Division;

use crate::{Scale, Scales};

pub(crate) const NAV_SCALE: &str = r#"{"features-and-maintenance/nav": [
    {"name": "lost", "meaning": "x", "parent": "critical"},
    {"name": "slow", "meaning": "x", "parent": "major"},
    {"name": "typo", "meaning": "x", "parent": "minor"}
]}"#;

impl Scales {
    pub(crate) fn declared(json: &str) -> Option<Scales> {
        Scales::try_from(serde_json::from_str::<BTreeMap<Division, Scale>>(json).ok()?).ok()
    }

    pub(crate) fn refusal(json: &str) -> Option<String> {
        Scales::try_from(serde_json::from_str::<BTreeMap<Division, Scale>>(json).ok()?)
            .err()
            .map(|failure| failure.to_string())
    }
}
