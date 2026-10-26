mod gh_jobs;
mod observation;
mod process_group;
mod raw_log;

use std::ffi::OsString;
use std::io::Read;
use std::os::unix::process::CommandExt as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::request::ci::ci_status::CiStatus;
use crate::request::ci::commit::Commit;
use crate::request::output::Listed;
use crate::request::project::ProgramName;
use gh_jobs::GhJobs;
use meta_gh::GhRun;
pub use observation::Observation;
use process_group::ProcessGroup;
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
        let mut child = Command::new(&self.program)
            .args(words)
            .current_dir(&self.root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(i32::default())
            .spawn()
            .map_err(Listed::new)?;
        let group = ProcessGroup::led_by(&child);
        let (stdout, stderr) = (child.stdout.take(), child.stderr.take());
        let (sender, drained) = std::sync::mpsc::channel::<(Vec<u8>, Vec<u8>)>();
        std::thread::spawn(move || {
            let errors = std::thread::spawn(move || Self::drain(stderr));
            let output = Self::drain(stdout);
            sender.send((output, errors.join().unwrap_or_default())).ok();
        });
        let early = drained.recv_timeout(self.deadline.saturating_duration_since(Instant::now()));
        early.is_err().then(|| group.kill());
        let (stdout, stderr) = early.or_else(|_| drained.recv()).map_err(Listed::new)?;
        let status = child.wait().map_err(Listed::new)?;
        if status.success() {
            Ok(String::from_utf8_lossy(&stdout).into_owned())
        } else {
            Err(Listed::new(String::from_utf8_lossy(&stderr).lines().next().unwrap_or_default()))
        }
    }

    fn drain(pipe: Option<impl Read>) -> Vec<u8> {
        let mut drained = Vec::new();
        pipe.map(|mut pipe| pipe.read_to_end(&mut drained).ok());
        drained
    }

    fn json<Parsed: serde::de::DeserializeOwned>(
        &self,
        words: impl Iterator<Item = OsString>,
    ) -> Result<Parsed, Listed> {
        self.text(words).and_then(|text| serde_json::from_str(&text).map_err(Listed::new))
    }
}
