use std::time::Duration;

use meta_gh::GhRun;

use super::commit_checks::CommitChecks;
use super::gh_jobs::GhJobs;
use super::gh_suite::GhSuite;
use crate::request::ci::ci_status::CiStatus;
use crate::request::output::Listed;

#[derive(Debug)]
pub struct Observation {
    failed_job: Option<Listed>,
    failed_steps: Vec<Listed>,
    status: Option<CiStatus>,
    expected: Option<Duration>,
}

impl Observation {
    pub fn new(
        failed_steps: Vec<Listed>,
        failed_job: Option<Listed>,
        status: Option<CiStatus>,
        expected: Option<Duration>,
    ) -> Self {
        Self { failed_job, failed_steps, status, expected }
    }

    pub fn failed_job(&self) -> Option<&Listed> {
        self.failed_job.as_ref()
    }

    pub fn failed_steps(&self) -> &[Listed] {
        &self.failed_steps
    }

    pub fn finished(&self) -> Option<CiStatus> {
        self.status.and_then(CiStatus::finished)
    }

    pub fn appeared(&self) -> Option<&Self> {
        self.status.map(|_| self)
    }

    pub fn expected(&self) -> Option<Duration> {
        self.expected
    }
}

impl From<&CommitChecks> for Observation {
    fn from(checks: &CommitChecks) -> Self {
        let suites: Vec<&GhSuite> = checks.actions();
        let runs: Vec<GhRun> = suites.iter().map(|suite| suite.run().clone()).collect();
        let jobs: Vec<&GhJobs> = suites.iter().map(|suite| suite.jobs()).collect();
        let refused = suites.iter().try_for_each(|suite| suite.run().failed().and(suite.unstarted()).map(|_| ()));
        let status = CiStatus::of_runs(&runs).map(|status| refused.map_or(status, |()| status.unstarted()));
        let reported = refused.map_or(jobs.as_slice(), |()| &[]);
        let failed_job = reported.iter().find_map(|jobs| jobs.first_failure());
        let failed_steps = reported.iter().flat_map(|jobs| jobs.failed_steps()).collect();
        Self::new(failed_steps, failed_job, status, checks.expected())
    }
}
