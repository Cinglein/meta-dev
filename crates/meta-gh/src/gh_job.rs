mod gh_step;
mod job_name;

use std::fmt;

use serde::Deserialize;

use crate::conclusion::Conclusion;
use crate::database_id::DatabaseId;
use gh_step::GhStep;
use job_name::JobName;

#[derive(Clone, Debug, Deserialize)]
pub struct GhJob {
    #[serde(default, rename = "databaseId")]
    database_id: DatabaseId,
    name: JobName,
    #[serde(default)]
    conclusion: Option<Conclusion>,
    #[serde(default)]
    steps: Vec<GhStep>,
}

impl GhJob {
    pub fn failure(&self) -> Option<&Self> {
        self.conclusion.and_then(Conclusion::failing).map(|_| self)
    }

    #[must_use]
    pub fn database_id(&self) -> DatabaseId {
        self.database_id
    }

    pub fn failed_steps(&self) -> Vec<String> {
        let steps: Vec<String> =
            self.steps.iter().filter_map(GhStep::failure).map(|step| format!("{self} / {step}")).collect();
        self.failure().map_or_else(Vec::new, |_| if steps.is_empty() { vec![self.to_string()] } else { steps })
    }
}

impl fmt::Display for GhJob {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
