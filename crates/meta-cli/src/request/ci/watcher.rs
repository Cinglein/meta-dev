mod elapsed;
mod gh;

use std::ops::ControlFlow;
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::ci_status::CiStatus;
use super::commit::Commit;
use super::session_dir::SessionDir;
use super::session_id::SessionId;
use crate::request::cli_failure::CliFailure;
use crate::request::output::{Listed, Output};
use crate::request::project::Project;
use elapsed::Elapsed;
use gh::{Gh, Observation};

#[derive(Debug)]
pub struct Watcher {
    dir: SessionDir,
    commit: Commit,
    gh: Gh,
    started: Instant,
}

impl Watcher {
    pub fn spawn(project: &Project, session: &SessionId, commit: &Commit) -> Option<()> {
        SessionDir::for_session(project, session)?.watch(commit).ok()?;
        Command::new(std::env::current_exe().ok()?)
            .args(["ci-watch", &session.to_string(), &commit.to_string()])
            .current_dir(project.root())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(i32::default())
            .spawn()
            .ok()
            .map(|_| ())
    }

    pub fn watch(project: &Project, session: &SessionId, commit: Commit) -> Result<Output, CliFailure> {
        let dir = SessionDir::for_session(project, session).ok_or_else(|| CliFailure::config("no git directory"))?;
        let settings = project.config().ci_settings();
        let gh = Gh::new(settings.program().clone(), project.root().to_path_buf());
        let watcher = Self { dir, commit, gh, started: Instant::now() };
        let interval = settings.poll_interval();
        let finished = std::iter::repeat_with(|| {
            std::thread::sleep(interval.duration());
            watcher.gh.observe(&watcher.commit)
        })
        .take(interval.polls_within(settings.timeout()))
        .try_fold(None, |reported, observed| watcher.step(reported, observed));
        watcher.conclude(finished.break_value())
    }

    pub fn wait(project: &Project, session: &SessionId) -> Result<Output, CliFailure> {
        let dir = SessionDir::for_session(project, session).ok_or_else(|| CliFailure::config("no git directory"))?;
        let interval = project.config().ci_settings().poll_interval();
        let settled = std::iter::once(Duration::ZERO)
            .chain(std::iter::repeat(interval.duration()))
            .take(interval.polls_within(project.config().ci_settings().timeout()))
            .find_map(|pause| {
                std::thread::sleep(pause);
                dir.status().and_then(CiStatus::finished)
            });
        let news = dir.news().map(|news| news.to_string()).unwrap_or_default();
        settled.map_or_else(
            || Ok(Output::Listed(Listed::new(format!("{news}\nCI still running; stopped waiting")))),
            |status| {
                status.failure().map_or_else(
                    || Ok(Output::Listed(Listed::new(format!("{news}\n")))),
                    |_| Err(CliFailure::invalid(&news)),
                )
            },
        )
    }

    pub fn failure_log(project: &Project, session: &SessionId) -> Result<Output, CliFailure> {
        let dir = SessionDir::for_session(project, session).ok_or_else(|| CliFailure::config("no git directory"))?;
        let commit = dir.tracked().ok_or_else(|| CliFailure::config("no CI tracked for this session"))?;
        let settings = project.config().ci_settings();
        let raw = Gh::new(settings.program().clone(), project.root().to_path_buf()).failed_logs(&commit);
        Ok(Output::Listed(Listed::new(format!("{}\n", raw.excerpt(settings.log_lines(), settings.log_context())))))
    }

    fn step(&self, reported: Option<Listed>, observed: Observation) -> ControlFlow<Observation, Option<Listed>> {
        let first_failure = observed.failed_job().filter(|_| reported.is_none());
        first_failure.iter().for_each(|job| {
            self.note(format!("{job} failed after {}; run `meta ci-log`", self.elapsed()), CiStatus::Failed);
        });
        let carried = reported.or_else(|| first_failure.map(Listed::new));
        match observed.finished() {
            Some(_) => ControlFlow::Break(observed),
            None => ControlFlow::Continue(carried),
        }
    }

    fn conclude(&self, last: Option<Observation>) -> Result<Output, CliFailure> {
        let status = last.as_ref().and_then(Observation::finished).unwrap_or(CiStatus::TimedOut);
        let steps: Vec<String> =
            last.iter().flat_map(|observed| observed.failed_steps()).map(ToString::to_string).collect();
        let failed = Some(steps.join(", "))
            .filter(|joined| !joined.is_empty())
            .map(|joined| format!(": {joined}; run `meta ci-log`"));
        self.note(format!("{status} after {}{}", self.elapsed(), failed.unwrap_or_default()), status);
        Ok(Output::Listed(Listed::new(status)))
    }

    fn note(&self, line: String, status: CiStatus) {
        self.dir.record(format!("CI {}: {line}", self.commit)).ok();
        self.dir.settle(&self.commit, status).ok();
    }

    fn elapsed(&self) -> Elapsed {
        Elapsed::new(self.started.elapsed())
    }
}

impl std::fmt::Display for Watcher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "CI watch of {} in {}", self.commit, self.dir)
    }
}
