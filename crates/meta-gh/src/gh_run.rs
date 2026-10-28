mod run_status;
mod workflow_run;

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::conclusion::Conclusion;
use crate::database_id::DatabaseId;
use crate::outcome::Outcome;
use run_status::RunStatus;
use workflow_run::WorkflowRun;

#[derive(Clone, Debug, Deserialize)]
pub struct GhRun {
    status: RunStatus,
    #[serde(default)]
    conclusion: Option<Conclusion>,
    #[serde(default, rename = "workflowRun")]
    workflow_run: Option<WorkflowRun>,
    #[serde(default, rename = "createdAt")]
    created: Option<DateTime<Utc>>,
    #[serde(default, rename = "updatedAt")]
    updated: Option<DateTime<Utc>>,
}

impl GhRun {
    #[must_use]
    pub fn database_id(&self) -> Option<DatabaseId> {
        self.workflow_run.map(WorkflowRun::database_id)
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

    #[must_use]
    pub fn took(&self) -> Option<Duration> {
        self.database_id()?;
        self.status.completed()?;
        self.updated?.signed_duration_since(self.created?).to_std().ok()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::GhRun;

    const TIMED: &str = r#"{"status":"COMPLETED","conclusion":"SUCCESS","workflowRun":{"databaseId":7},"createdAt":"2026-10-27T10:00:00Z","updatedAt":"2026-10-27T10:00:05Z"}"#;
    const RUNNING: &str = r#"{"status":"IN_PROGRESS","workflowRun":{},"createdAt":"2026-10-27T10:00:00Z"}"#;
    const OTHER_APP: &str = r#"{"status":"COMPLETED","workflowRun":null,"createdAt":"2026-10-27T10:00:00Z","updatedAt":"2026-10-27T10:00:05Z"}"#;
    const FIVE_SECONDS: Duration = Duration::from_secs(5);
    const RUN_ID: &str = "7";

    #[test]
    fn only_a_finished_actions_run_tells_how_long_it_took() {
        let runs = [TIMED, RUNNING, OTHER_APP].map(|text| serde_json::from_str::<GhRun>(text).unwrap());
        assert_eq!(runs.each_ref().map(GhRun::took), [Some(FIVE_SECONDS), None, None]);
        assert_eq!(runs[0].database_id(), serde_json::from_str(RUN_ID).ok());
    }
}
