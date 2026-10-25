mod gh_job;

use std::ffi::OsString;

use serde::Deserialize;

use crate::command::output::Listed;
use gh_job::GhJob;

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
        self.jobs.iter().flat_map(GhJob::failed_steps).collect()
    }

    pub fn failed_job_ids(&self) -> Vec<OsString> {
        self.jobs.iter().filter_map(GhJob::failure).map(GhJob::job_id).collect()
    }
}
