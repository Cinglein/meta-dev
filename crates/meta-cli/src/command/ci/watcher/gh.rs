mod gh_jobs;
mod gh_run;
mod observation;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use crate::command::ci::commit::Commit;
use crate::command::project::ProgramName;
use gh_jobs::GhJobs;
use gh_run::GhRun;
pub use observation::Observation;

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

    pub fn new(program: ProgramName, root: PathBuf) -> Self {
        Self { program, root }
    }

    pub fn observe(&self, commit: &Commit) -> Observation {
        let list = [Self::RUN, Self::LIST, Self::COMMIT]
            .map(OsString::from)
            .into_iter()
            .chain([OsString::from(commit.to_string())]);
        let runs: Vec<GhRun> =
            self.json(list.chain([Self::JSON, Self::RUN_FIELDS].map(OsString::from))).unwrap_or_default();
        let failed_job = runs
            .iter()
            .filter_map(|run| {
                let view = [Self::RUN, Self::VIEW].map(OsString::from).into_iter().chain([run.run_id()]);
                self.json::<GhJobs>(view.chain([Self::JSON, Self::JOB_FIELDS].map(OsString::from)))
            })
            .find_map(|jobs| jobs.first_failure());
        Observation::new(failed_job, GhRun::overall(&runs))
    }

    fn json<Parsed: serde::de::DeserializeOwned>(&self, words: impl Iterator<Item = OsString>) -> Option<Parsed> {
        let output = Command::new(&self.program).args(words).current_dir(&self.root).output().ok()?;
        output.status.success().then(|| serde_json::from_slice(&output.stdout).ok()).flatten()
    }
}

impl std::fmt::Display for Gh {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} in {}", self.program, self.root.display())
    }
}
