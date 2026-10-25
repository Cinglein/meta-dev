mod run_status;

use std::fmt;

use serde::Deserialize;

use super::conclusion::Conclusion;
use super::database_id::DatabaseId;
use crate::command::ci::ci_status::CiStatus;
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

    pub fn overall(runs: &[Self]) -> Option<CiStatus> {
        runs.first().filter(|_| runs.iter().all(|run| run.status.completed().is_some())).map(|_| {
            runs.iter()
                .find(|run| run.conclusion.and_then(Conclusion::succeeded).is_none())
                .map_or(CiStatus::Passed, |_| CiStatus::Failed)
        })
    }
}

impl fmt::Display for GhRun {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "run {} {}", self.database_id, self.status)
    }
}
