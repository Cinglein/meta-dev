mod gh_step;

use std::ffi::OsString;
use std::fmt;

use serde::Deserialize;

use crate::command::output::Listed;
use gh_step::GhStep;

#[derive(Clone, Debug, Deserialize)]
pub struct GhJob {
    #[serde(default, rename = "databaseId")]
    database_id: serde_json::Value,
    name: serde_json::Value,
    #[serde(default)]
    conclusion: serde_json::Value,
    #[serde(default)]
    steps: Vec<GhStep>,
}

impl GhJob {
    const FAILING: &'static [&'static str] = &["failure", "timed_out", "cancelled", "startup_failure"];

    pub fn failure(&self) -> Option<&Self> {
        Self::FAILING.iter().find(|failing| self.conclusion == **failing).map(|_| self)
    }

    pub fn job_id(&self) -> OsString {
        OsString::from(self.database_id.to_string())
    }

    pub fn failed_steps(&self) -> Vec<Listed> {
        let steps: Vec<Listed> =
            self.steps.iter().filter_map(GhStep::failure).map(|step| Listed::new(format!("{self} / {step}"))).collect();
        self.failure().map_or_else(Vec::new, |_| if steps.is_empty() { vec![Listed::new(self)] } else { steps })
    }
}

impl fmt::Display for GhJob {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.name.as_str().unwrap_or_default())
    }
}
