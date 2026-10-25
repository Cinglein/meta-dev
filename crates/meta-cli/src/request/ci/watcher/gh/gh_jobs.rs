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

    pub fn failed_steps(&self) -> Vec<Listed> {
        self.jobs.iter().flat_map(GhJob::failed_steps).map(Listed::new).collect()
    }

    pub fn failed_job_ids(&self) -> Vec<OsString> {
        self.jobs.iter().filter_map(GhJob::failure).map(|job| job.database_id().argument()).collect()
    }
}
