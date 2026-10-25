use std::ffi::OsString;
use std::fmt;

use serde::Deserialize;

use crate::command::ci::ci_status::CiStatus;

#[derive(Clone, Debug, Deserialize)]
pub struct GhRun {
    #[serde(rename = "databaseId")]
    database_id: serde_json::Value,
    status: serde_json::Value,
    #[serde(default)]
    conclusion: serde_json::Value,
}

impl GhRun {
    const COMPLETED: &'static str = "completed";
    const SUCCESS: &'static str = "success";

    pub fn run_id(&self) -> OsString {
        OsString::from(self.database_id.to_string())
    }

    pub fn overall(runs: &[Self]) -> Option<CiStatus> {
        runs.first().filter(|_| runs.iter().all(|run| run.status == Self::COMPLETED)).map(|_| {
            runs.iter().find(|run| run.conclusion != Self::SUCCESS).map_or(CiStatus::Passed, |_| CiStatus::Failed)
        })
    }
}

impl fmt::Display for GhRun {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "run {} {} {}", self.database_id, self.status, self.conclusion)
    }
}
