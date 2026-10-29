mod commit_checks;
mod gh_jobs;
mod gh_suite;
mod observation;
mod raw_log;

use std::ffi::OsString;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::request::ci::commit::Commit;
use crate::request::output::Listed;
use crate::request::project::{CiSettings, LogWindow};
use crate::request::repository::Repository;
use commit_checks::CommitChecks;
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
        self.checks(repository, commit).map(|checks| Observation::from(&checks))
    }

    pub fn failed_excerpt(&self, repository: &Repository, commits: &[Commit], window: &LogWindow) -> Listed {
        let checks: Vec<Result<CommitChecks, Listed>> =
            commits.iter().map(|commit| self.checks(repository, commit)).collect();
        let jobs: Vec<OsString> = checks.iter().flatten().flat_map(CommitChecks::failed_job_ids).collect();
        let logs: Vec<Result<String, Listed>> = jobs.into_iter().map(|job| self.download(job)).collect();
        Self::excerpt(&checks, &logs, window)
    }

    pub fn excerpt(
        checks: &[Result<CommitChecks, Listed>],
        logs: &[Result<String, Listed>],
        window: &LogWindow,
    ) -> Listed {
        let text: String = logs.iter().flatten().map(String::as_str).collect();
        let troubles = checks.iter().filter_map(|found| found.as_ref().err());
        let troubles = troubles.chain(logs.iter().filter_map(|log| log.as_ref().err()));
        let Ok(raw): Result<RawLog, _> = text.parse();
        troubles.fold(raw.excerpt(window), |shown, trouble| shown.then("\n").then(trouble))
    }

    pub fn checks_in(reply: &str) -> Result<CommitChecks, Listed> {
        let found: Option<CommitChecks> = serde_json::from_str(reply).map_err(Self::failed)?;
        Ok(found.unwrap_or_default())
    }

    pub fn deadline_of(&self, call: GhArgv, now: Instant) -> Instant {
        match call {
            GhArgv::CommitChecks => self.deadline.min(now + self.limit),
            GhArgv::FailedLog => self.deadline,
        }
    }

    pub fn failed(trouble: impl std::fmt::Display) -> Listed {
        Listed::new(Self::FAILED).then(trouble)
    }

    fn download(&self, job: OsString) -> Result<String, Listed> {
        let deadline = self.deadline_of(GhArgv::FailedLog, Instant::now());
        let log = self.port.reported(GhArgv::FailedLog, [job], deadline).map_err(Self::failed)?;
        Ok(log.text().into_owned())
    }

    fn checks(&self, repository: &Repository, commit: &Commit) -> Result<CommitChecks, Listed> {
        let deadline = self.deadline_of(GhArgv::CommitChecks, Instant::now());
        let text = self.port.reported(GhArgv::CommitChecks, repository.variables(commit), deadline);
        Self::checks_in(&text.map_err(Self::failed)?.text())
    }
}

#[cfg(test)]
pub mod replies;
#[cfg(test)]
mod tests;
