mod run_status;

use serde::Deserialize;

use crate::conclusion::Conclusion;
use crate::database_id::DatabaseId;
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
    pub fn database_id(&self) -> DatabaseId {
        self.database_id
    }

    pub fn completed(&self) -> Option<&Self> {
        self.status.completed().map(|_| self)
    }

    pub fn succeeded(&self) -> Option<&Self> {
        self.conclusion.and_then(Conclusion::succeeded).map(|_| self)
    }
}
