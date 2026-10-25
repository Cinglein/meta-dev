use std::fmt;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct GhStep {
    name: serde_json::Value,
    #[serde(default)]
    conclusion: serde_json::Value,
}

impl GhStep {
    const FAILURE: &'static str = "failure";

    pub fn failure(&self) -> Option<&Self> {
        (self.conclusion == Self::FAILURE).then_some(self)
    }
}

impl fmt::Display for GhStep {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name.as_str().unwrap_or_default())
    }
}
