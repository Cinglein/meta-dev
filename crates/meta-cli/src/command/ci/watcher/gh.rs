mod gh_jobs;
mod gh_run;
mod observation;
mod raw_log;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use crate::command::ci::commit::Commit;
use crate::command::project::ProgramName;
use gh_jobs::GhJobs;
use gh_run::GhRun;
pub use observation::Observation;
pub use raw_log::RawLog;

#[derive(Clone, Debug)]
pub struct Gh {
    program: ProgramName,
    root: PathBuf,
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

    pub fn new(program: ProgramName, root: PathBuf) -> Self {
        Self { program, root }
    }

    pub fn observe(&self, commit: &Commit) -> Observation {
        let runs = self.runs(commit);
        let jobs: Vec<GhJobs> = runs.iter().filter_map(|run| self.jobs(run)).collect();
        let failed_job = jobs.iter().find_map(GhJobs::first_failure);
        Observation::new(jobs.iter().flat_map(GhJobs::failed_steps).collect(), failed_job, GhRun::overall(&runs))
    }

    pub fn failed_logs(&self, commit: &Commit) -> RawLog {
        let logs: Vec<String> = self
            .runs(commit)
            .iter()
            .filter_map(|run| self.jobs(run))
            .flat_map(|jobs| jobs.failed_job_ids())
            .filter_map(|job| {
                let words = [Self::RUN, Self::VIEW, Self::JOB].map(OsString::from).into_iter().chain([job]);
                self.text(words.chain([OsString::from(Self::LOG_FAILED)]))
            })
            .collect();
        RawLog::new(logs.concat())
    }

    fn runs(&self, commit: &Commit) -> Vec<GhRun> {
        let list = [Self::RUN, Self::LIST, Self::COMMIT]
            .map(OsString::from)
            .into_iter()
            .chain([OsString::from(commit.to_string())]);
        self.json(list.chain([Self::JSON, Self::RUN_FIELDS].map(OsString::from))).unwrap_or_default()
    }

    fn jobs(&self, run: &GhRun) -> Option<GhJobs> {
        let view = [Self::RUN, Self::VIEW].map(OsString::from).into_iter().chain([run.run_id()]);
        self.json(view.chain([Self::JSON, Self::JOB_FIELDS].map(OsString::from)))
    }

    fn text(&self, words: impl Iterator<Item = OsString>) -> Option<String> {
        let output = Command::new(&self.program).args(words).current_dir(&self.root).output().ok()?;
        output.status.success().then(|| String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn json<Parsed: serde::de::DeserializeOwned>(&self, words: impl Iterator<Item = OsString>) -> Option<Parsed> {
        self.text(words).and_then(|text| serde_json::from_str(&text).ok())
    }
}

impl std::fmt::Display for Gh {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} in {}", self.program, self.root.display())
    }
}
