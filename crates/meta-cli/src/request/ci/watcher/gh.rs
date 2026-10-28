mod commit_checks;
mod gh_jobs;
mod gh_suite;
mod observation;
mod raw_log;

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::request::ci::ci_status::CiStatus;
use crate::request::ci::commit::Commit;
use crate::request::output::Listed;
use crate::request::project::{CiSettings, LogWindow, ProgramName};
use crate::request::repository::Repository;
use commit_checks::CommitChecks;
use gh_jobs::GhJobs;
use gh_suite::GhSuite;
use meta_gh::GhRun;
use meta_run::{EnvPolicy, Run};
pub use observation::Observation;
pub use raw_log::RawLog;

#[derive(Clone, Debug)]
pub struct Gh {
    program: ProgramName,
    root: PathBuf,
    deadline: Instant,
    limit: Duration,
}

impl Gh {
    const GRAPHQL: [&'static str; 2] = ["api", "graphql"];
    const FIELD: &'static str = "-f";
    const QUERY: &'static str = "query=query($owner:String!,$name:String!,$sha:GitObjectID!){repository(owner:$owner,name:$name){object(oid:$sha){...on Commit{checkSuites(first:20,filterBy:{appId:15368}){nodes{status conclusion workflowRun{databaseId} checkRuns(first:100,filterBy:{conclusions:[FAILURE,CANCELLED,TIMED_OUT,STARTUP_FAILURE,ACTION_REQUIRED]}){nodes{databaseId name conclusion steps(first:100){nodes{name conclusion}}}} otherJobs:checkRuns(first:1,filterBy:{conclusions:[SUCCESS,NEUTRAL,SKIPPED,STALE]}){nodes{databaseId name}}}} history(first:8){nodes{checkSuites(first:20,filterBy:{appId:15368}){nodes{status conclusion workflowRun{databaseId} createdAt updatedAt}}}}}}}}";
    const OBJECT: [&'static str; 2] = ["--jq", ".data.repository.object"];
    const VIEW_JOB: [&'static str; 3] = ["run", "view", "--job"];
    const LOG_FAILED: &'static str = "--log-failed";
    const FAILED: &'static str = "gh failed: ";

    pub fn new(settings: &CiSettings, root: PathBuf, deadline: Instant) -> Self {
        Self { program: settings.program().clone(), root, deadline, limit: settings.gh_limit() }
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
        let words = Self::VIEW_JOB.map(OsString::from).into_iter().chain([job, OsString::from(Self::LOG_FAILED)]);
        self.run(words, self.deadline)
    }

    fn checks(&self, repository: &Repository, commit: &Commit) -> Result<CommitChecks, Listed> {
        let query = [Self::FIELD, Self::QUERY].map(OsString::from);
        let words = Self::GRAPHQL.map(OsString::from).into_iter().chain(repository.variables(commit));
        let words = words.chain(query).chain(Self::OBJECT.map(OsString::from));
        let text = self.run(words, self.deadline.min(Instant::now() + self.limit))?;
        let found: Option<CommitChecks> = serde_json::from_str(&text).map_err(Self::failed)?;
        Ok(found.unwrap_or_default())
    }

    fn failed(trouble: impl std::fmt::Display) -> Listed {
        Listed::new(Self::FAILED).then(trouble)
    }

    fn run(&self, words: impl Iterator<Item = OsString>, deadline: Instant) -> Result<String, Listed> {
        let run = Run::new(&self.program, EnvPolicy::Inherit).args(words).dir(&self.root);
        run.stdout(deadline).map_err(Self::failed)
    }
}

#[cfg(test)]
mod tests;
