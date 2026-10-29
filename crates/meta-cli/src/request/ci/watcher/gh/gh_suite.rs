use meta_gh::GhRun;
use serde::Deserialize;

use super::gh_jobs::GhJobs;

#[derive(Clone, Debug, Deserialize)]
pub struct GhSuite {
    #[serde(flatten)]
    run: GhRun,
    #[serde(default, rename = "checkRuns")]
    jobs: GhJobs,
    #[serde(default, rename = "otherJobs")]
    others: GhJobs,
}

impl GhSuite {
    pub fn of_actions(&self) -> Option<&Self> {
        self.run.database_id().map(|_| self)
    }

    pub fn run(&self) -> &GhRun {
        &self.run
    }

    pub fn jobs(&self) -> &GhJobs {
        &self.jobs
    }

    pub fn unstarted(&self) -> Option<&GhJobs> {
        self.jobs.unstarted().filter(|_| self.others.is_empty())
    }
}
