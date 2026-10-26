use std::ffi::OsString;

use meta_gh::GhJob;
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

    pub fn unstarted(&self) -> Option<&Self> {
        self.jobs.first()?;
        self.jobs.iter().all(|job| job.unstarted().is_some()).then_some(self)
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
    const EMPTY: &str = r#"{"jobs":[]}"#;

    #[test]
    fn a_failed_job_without_steps_never_started() {
        let started: GhJobs = serde_json::from_str(STARTED).unwrap();
        let unstarted: GhJobs = serde_json::from_str(UNSTARTED).unwrap();
        let empty: GhJobs = serde_json::from_str(EMPTY).unwrap();
        assert!(started.unstarted().is_none() && unstarted.unstarted().is_some() && empty.unstarted().is_none());
    }
}
