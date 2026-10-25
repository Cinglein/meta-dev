use std::fmt;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct GhJob {
    name: serde_json::Value,
    #[serde(default)]
    conclusion: serde_json::Value,
}

impl GhJob {
    const FAILING: &'static [&'static str] = &["failure", "timed_out", "cancelled", "startup_failure"];

    pub fn failure(&self) -> Option<&Self> {
        Self::FAILING.iter().find(|failing| self.conclusion == **failing).map(|_| self)
    }
}

impl fmt::Display for GhJob {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.name.as_str().unwrap_or_default())
    }
}
