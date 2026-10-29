use serde::{Deserialize, Serialize};
use settings::EmbeddedDefaults;

use crate::intent_cap::IntentCap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EleganceSettings {
    max_intents: IntentCap,
}

impl EmbeddedDefaults for EleganceSettings {
    const DEFAULTS: &str = include_str!("elegance_defaults.toml");
}

impl EleganceSettings {
    #[must_use]
    pub fn max_intents(&self) -> IntentCap {
        self.max_intents
    }
}
