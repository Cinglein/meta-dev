mod gh_jobs;
mod observation;
mod raw_log;

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Instant;

use crate::request::ci::ci_status::CiStatus;
use crate::request::ci::commit::Commit;
use crate::request::output::Listed;
use crate::request::project::ProgramName;
use gh_jobs::GhJobs;
use meta_gh::GhRun;
use meta_run::{EnvPolicy, KillScope, Run};
pub use observation::Observation;
pub use raw_log::RawLog;

#[derive(Clone, Debug)]
pub struct Gh {
    program: ProgramName,
    root: PathBuf,
    deadline: Instant,
}

impl Gh {
    const RUN: &'static str = "run";
    const LIST: &'static str = "list";
    const VIEW: &'static str = "view";
    const COMMIT: &'static str = "--commit";
    const JSON: &'static str = "--json";
    const RUN_FIELDS: &'static str = "databaseId,status,conclusion";
    const JOB_FIELDS: &'static str = "jobs";
    const JOB: &'static str = "--job";
    const LOG_FAILED: &'static str = "--log-failed";

    pub fn new(program: ProgramName, root: PathBuf, deadline: Instant) -> Self {
        Self { program, root, deadline }
    }

    pub fn observe(&self, commit: &Commit) -> Result<Observation, Listed> {
        let runs = self.runs(commit)?;
        let jobs = runs.iter().map(|run| self.jobs(run)).collect::<Result<Vec<GhJobs>, Listed>>()?;
        let refused = runs.iter().zip(&jobs).try_for_each(|(run, jobs)| run.failed().and(jobs.unstarted()).map(|_| ()));
        let status = CiStatus::of_runs(&runs).map(|status| refused.map_or(status, |()| status.unstarted()));
        let reported = refused.map_or(jobs.as_slice(), |()| &[]);
        let failed_job = reported.iter().find_map(GhJobs::first_failure);
        Ok(Observation::new(reported.iter().flat_map(GhJobs::failed_steps).collect(), failed_job, status))
    }

    pub fn failed_logs(&self, commits: &[Commit]) -> RawLog {
        let logs: Vec<String> = commits
            .iter()
            .flat_map(|commit| self.runs(commit).unwrap_or_default())
            .filter_map(|run| self.jobs(&run).ok())
            .flat_map(|jobs| jobs.failed_job_ids())
            .filter_map(|job| {
                let words = [Self::RUN, Self::VIEW, Self::JOB].map(OsString::from).into_iter().chain([job]);
                self.text(words.chain([OsString::from(Self::LOG_FAILED)])).ok()
            })
            .collect();
        let Ok(raw) = logs.concat().parse();
        raw
    }

    fn runs(&self, commit: &Commit) -> Result<Vec<GhRun>, Listed> {
        let list = [Self::RUN, Self::LIST, Self::COMMIT].map(OsString::from).into_iter().chain([commit.argument()]);
        self.json(list.chain([Self::JSON, Self::RUN_FIELDS].map(OsString::from)))
    }

    fn jobs(&self, run: &GhRun) -> Result<GhJobs, Listed> {
        let view = [Self::RUN, Self::VIEW].map(OsString::from).into_iter().chain([run.database_id().argument()]);
        self.json(view.chain([Self::JSON, Self::JOB_FIELDS].map(OsString::from)))
    }

    fn text(&self, words: impl Iterator<Item = OsString>) -> Result<String, Listed> {
        let run = Run::new(&self.program, EnvPolicy::Inherit).args(words).dir(&self.root).scoped(KillScope::Group);
        run.stdout(self.deadline).map_err(Listed::new)
    }

    fn json<Parsed: serde::de::DeserializeOwned>(
        &self,
        words: impl Iterator<Item = OsString>,
    ) -> Result<Parsed, Listed> {
        self.text(words).and_then(|text| serde_json::from_str(&text).map_err(Listed::new))
    }
}
