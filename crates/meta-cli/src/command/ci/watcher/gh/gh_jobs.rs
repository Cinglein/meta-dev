mod gh_job;

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
}
