use std::ffi::OsString;

use meta_gh::{DatabaseId, GhJob};
use serde::Deserialize;

use crate::request::output::Listed;

#[derive(Clone, Debug, Deserialize)]
pub struct GhJobs {
    #[serde(default)]
    jobs: Vec<GhJob>,
}

impl GhJobs {
    pub fn first_failure(&self) -> Option<Listed> {
        self.jobs.iter().find_map(GhJob::failure).map(Listed::new)
    }

    pub fn first_unstarted(&self) -> Option<DatabaseId> {
        self.jobs.iter().find_map(GhJob::unstarted)
    }

    pub fn failed_steps(&self) -> Vec<Listed> {
        self.jobs.iter().flat_map(GhJob::failed_steps).map(Listed::new).collect()
    }

    pub fn failed_job_ids(&self) -> Vec<OsString> {
        self.jobs.iter().filter_map(GhJob::failure).map(|job| job.database_id().argument()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::GhJobs;

    const STARTED: &str =
        r#"{"jobs":[{"name":"ci","conclusion":"failure","steps":[{"name":"t","conclusion":"failure"}]}]}"#;
    const UNSTARTED: &str = r#"{"jobs":[{"name":"ci","conclusion":"failure","steps":[]}]}"#;

    #[test]
    fn a_failed_job_without_steps_never_started() {
        let started: GhJobs = serde_json::from_str(STARTED).unwrap();
        let unstarted: GhJobs = serde_json::from_str(UNSTARTED).unwrap();
        assert!(started.first_unstarted().is_none() && started.first_failure().is_some());
        assert!(unstarted.first_unstarted().is_some() && unstarted.first_failure().is_none());
    }
}
