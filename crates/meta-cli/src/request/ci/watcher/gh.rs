mod commit_checks;
mod gh_jobs;
mod gh_suite;
mod observation;
mod raw_log;

use std::ffi::OsString;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::request::ci::ci_status::CiStatus;
use crate::request::ci::commit::Commit;
use crate::request::output::Listed;
use crate::request::project::{CiSettings, LogWindow};
use crate::request::repository::Repository;
use commit_checks::CommitChecks;
use gh_jobs::GhJobs;
use gh_suite::GhSuite;
use meta_gh::GhRun;
use meta_run::{GhArgv, Port};
pub use observation::Observation;
pub use raw_log::RawLog;

#[derive(Clone, Debug)]
pub struct Gh {
    port: Port<GhArgv>,
    deadline: Instant,
    limit: Duration,
}

impl Gh {
    const FAILED: &'static str = "gh failed: ";

    pub fn new(settings: &CiSettings, root: &Path, deadline: Instant) -> Self {
        Self { port: Port::named(settings.program(), root), deadline, limit: settings.gh_limit() }
    }

    pub fn observe(&self, repository: &Repository, commit: &Commit) -> Result<Observation, Listed> {
        let checks = self.checks(repository, commit)?;
        let suites: Vec<&GhSuite> = checks.actions();
        let runs: Vec<GhRun> = suites.iter().map(|suite| suite.run().clone()).collect();
        let jobs: Vec<&GhJobs> = suites.iter().map(|suite| suite.jobs()).collect();
        let refused = suites.iter().try_for_each(|suite| suite.run().failed().and(suite.unstarted()).map(|_| ()));
        let status = CiStatus::of_runs(&runs).map(|status| refused.map_or(status, |()| status.unstarted()));
        let reported = refused.map_or(jobs.as_slice(), |()| &[]);
        let failed_job = reported.iter().find_map(|jobs| jobs.first_failure());
        let failed_steps = reported.iter().flat_map(|jobs| jobs.failed_steps()).collect();
        Ok(Observation::new(failed_steps, failed_job, status, checks.expected()))
    }

    pub fn failed_excerpt(&self, repository: &Repository, commits: &[Commit], window: &LogWindow) -> Listed {
        let checks: Vec<Result<CommitChecks, Listed>> =
            commits.iter().map(|commit| self.checks(repository, commit)).collect();
        let jobs: Vec<OsString> = checks.iter().flatten().flat_map(CommitChecks::failed_job_ids).collect();
        let logs: Vec<Result<String, Listed>> = jobs.into_iter().map(|job| self.download(job)).collect();
        let text: String = logs.iter().flatten().map(String::as_str).collect();
        let troubles = checks.iter().filter_map(|found| found.as_ref().err());
        let troubles = troubles.chain(logs.iter().filter_map(|log| log.as_ref().err()));
        let Ok(raw): Result<RawLog, _> = text.parse();
        troubles.fold(raw.excerpt(window), |shown, trouble| shown.then("\n").then(trouble))
    }

    fn download(&self, job: OsString) -> Result<String, Listed> {
        let log = self.port.reported(GhArgv::FailedLog, [job], self.deadline).map_err(Self::failed)?;
        Ok(log.text().into_owned())
    }

    fn checks(&self, repository: &Repository, commit: &Commit) -> Result<CommitChecks, Listed> {
        let deadline = self.deadline.min(Instant::now() + self.limit);
        let text = self.port.reported(GhArgv::CommitChecks, repository.variables(commit), deadline);
        let found: Option<CommitChecks> =
            serde_json::from_str(&text.map_err(Self::failed)?.text()).map_err(Self::failed)?;
        Ok(found.unwrap_or_default())
    }

    fn failed(trouble: impl std::fmt::Display) -> Listed {
        Listed::new(Self::FAILED).then(trouble)
    }
}

#[cfg(test)]
mod tests;
