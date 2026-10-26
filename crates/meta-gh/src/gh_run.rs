mod run_status;

use serde::Deserialize;

use crate::conclusion::Conclusion;
use crate::database_id::DatabaseId;
use crate::outcome::Outcome;
use run_status::RunStatus;

#[derive(Clone, Debug, Deserialize)]
pub struct GhRun {
    #[serde(rename = "databaseId")]
    database_id: DatabaseId,
    status: RunStatus,
    #[serde(default)]
    conclusion: Option<Conclusion>,
}

impl GhRun {
    #[must_use]
    pub fn database_id(&self) -> DatabaseId {
        self.database_id
    }

    #[must_use]
    pub fn failed(&self) -> Option<&Self> {
        self.status
            .completed()
            .and(self.conclusion)
            .filter(|conclusion| *conclusion == Conclusion::Failure)
            .map(|_| self)
    }

    #[must_use]
    pub fn outcome(&self) -> Option<Outcome> {
        self.status.completed().map(|_| self.conclusion.map_or(Outcome::Undecided, Conclusion::outcome))
    }
}
