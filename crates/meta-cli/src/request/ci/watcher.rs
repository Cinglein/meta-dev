mod elapsed;
mod failure_log;
mod gh;
mod reach;
mod session_commits;
mod wait_on_ci;
mod watch_clock;

use std::cell::OnceCell;
use std::ops::ControlFlow;
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::ci_status::CiStatus;
use super::commit::Commit;
use super::session_dir::SessionDir;
use super::session_id::SessionId;
use crate::request::cli_failure::{CliFailure, Invalid, Setup};
use crate::request::output::{Listed, Output};
use crate::request::project::{CiSettings, Project};
use elapsed::Elapsed;
pub use failure_log::FailureLog;
use gh::{Gh, Observation};
use reach::Reach;
use session_commits::SessionCommits;
use wait_on_ci::WaitOnCi;
use watch_clock::WatchClock;

#[derive(Debug)]
pub struct Watcher {
    dir: SessionDir,
    commit: Commit,
    gh: Gh,
    clock: WatchClock,
    settings: CiSettings,
    appeared: OnceCell<()>,
}

impl Watcher {
    const NO_RUN_YET: &'static str = "no run yet; Stop stays blocked until one appears or the wait is recorded";
    const RUN_APPEARED: &'static str = "run appeared";
    const OUT_OF_TIME: &'static str = "couldn't check CI in time; run `meta ci-wait`, then stop again";

    pub fn spawn(project: &Project, session: &SessionId, commit: &Commit) -> Result<(), Listed> {
        let unwatched = |error: std::io::Error| Listed::new(format!("CI not watched: {error}"));
        let dir =
            SessionDir::for_session(project, session).ok_or_else(|| Listed::new("CI not watched: no git directory"))?;
        dir.writable().map_err(unwatched)?;
        let status = dir.status_of(commit).map_or_else(|| dir.watch(commit).map(|()| CiStatus::Running), Ok);
        let idle = status.map_err(unwatched)?.finished().is_none().then(|| dir.claim(commit).map(drop)).flatten();
        idle.map_or(Ok(()), |()| {
            Command::new(std::env::current_exe().map_err(unwatched)?)
                .args(["ci-watch", &session.to_string(), &commit.to_string()])
                .current_dir(project.root())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .process_group(i32::default())
                .spawn()
                .map(|_| ())
                .map_err(unwatched)
        })
    }

    pub fn watch_all(project: &Project, session: &SessionId) -> Result<(), Listed> {
        let deadline = project.config().ci_settings().remote_deadline();
        let seen = SessionCommits::judged(project, session, Reach::Tracking, deadline)?;
        seen.pushed().iter().try_for_each(|commit| Self::spawn(project, session, commit))
    }

    pub fn stop_reason(project: &Project, session: &SessionId) -> Option<Listed> {
        let dir = SessionDir::for_session(project, session)?;
        let limit = CiSettings::try_from(project.config()).map(|settings| settings.stop_limit().deadline());
        let deadline = limit.as_ref().ok().copied();
        let seen = limit
            .map_err(|refused| Listed::new(format!("{refused}; fix it or ask the owner")))
            .and_then(|deadline| SessionCommits::judged(project, session, Reach::Remote, deadline));
        let unknown = seen.as_ref().err().map(|reason| Listed::new(reason.to_string()));
        let commits = seen.as_ref().map(|seen| seen.pushed().to_vec()).unwrap_or_default();
        let unwritable = commits
            .first()
            .and(dir.writable().err())
            .map(|error| Listed::new(format!("CI state unwritable: {error}; fix it or ask the owner")));
        let unpushed = seen.as_ref().ok().and_then(SessionCommits::unpushed_head).map(|commit| {
            Listed::new(format!("HEAD {commit} is not pushed: push it, then run `meta ci-wait`; or ask the owner"))
        });
        let statuses: Vec<(Commit, Option<CiStatus>)> = commits
            .iter()
            .map(|commit| {
                Self::spawn(project, session, commit).ok();
                (commit.clone(), dir.status_of(commit))
            })
            .collect();
        let unwatched = statuses
            .iter()
            .find(|(_, status)| status.is_none())
            .map(|(commit, _)| Listed::new(format!("CI not watched for {commit}: push again to watch it")));
        let known = statuses.iter().find_map(|(_, status)| status.and_then(CiStatus::stop_reason));
        let judged = seen.as_ref().ok().zip(deadline);
        let unresolved = || judged.and_then(|(seen, deadline)| WaitOnCi::stop_reason(project, &dir, seen, deadline));
        let reason = unknown.or(unwritable).or(unpushed).or(unwatched).or(known).or_else(unresolved);
        let late = deadline.filter(|deadline| Instant::now() >= *deadline).map(|_| Listed::new(Self::OUT_OF_TIME));
        late.or(reason).map(|reason| Listed::new(format!("{reason}{}", WatchClock::mark())))
    }

    pub fn watch(project: &Project, session: &SessionId, commit: Commit) -> Result<Output, CliFailure> {
        let dir =
            SessionDir::for_session(project, session).ok_or_else(|| CliFailure::raised::<Setup>("no git directory"))?;
        let _lock = dir.claim(&commit).ok_or_else(|| CliFailure::raised::<Invalid>("another watcher has it"))?;
        let settled = dir.status_of(&commit).and_then(CiStatus::finished);
        settled.map_or(Ok(()), |status| Err(CliFailure::raised::<Invalid>(format!("already {status}"))))?;
        let settings = project.config().ci_settings();
        let repository = project.repository().ok_or_else(|| {
            let status = CiStatus::Unchecked;
            dir.record(format!("CI {commit}{}: {status} after 0s{}", WatchClock::mark(), status.detail())).ok();
            dir.settle(&commit, status).ok();
            CliFailure::raised::<Setup>("no remote URL")
        })?;
        let clock = WatchClock::start(&settings);
        let gh = Gh::new(&settings, project.root().to_path_buf(), clock.deadline(), repository);
        let waits = settings.poll_schedule().waits(dir.expected());
        let watcher = Self { dir, commit, gh, clock, settings: settings.clone(), appeared: OnceCell::new() };
        let finished = waits
            .map_while(|wait| {
                watcher.clock.pause(wait, &settings)?;
                let observed = watcher.gh.observe(&watcher.commit);
                let learned = observed.as_ref().ok().and_then(Observation::expected);
                learned.map(|expected| watcher.dir.learn(expected).ok());
                Some(observed)
            })
            .try_fold(None, |reported, observed| watcher.step(reported, observed));
        Ok(watcher.conclude(finished.break_value().as_ref()))
    }

    pub fn wait(project: &Project, session: &SessionId) -> Result<Output, CliFailure> {
        let dir =
            SessionDir::for_session(project, session).ok_or_else(|| CliFailure::raised::<Setup>("no git directory"))?;
        let settings = project.config().ci_settings();
        let seen = SessionCommits::judged(project, session, Reach::Remote, settings.remote_deadline())
            .map_err(CliFailure::raised::<Invalid>)?;
        let pushed = seen.pushed();
        pushed.first().ok_or_else(|| CliFailure::raised::<Invalid>("no CI watched for this session"))?;
        pushed
            .iter()
            .try_for_each(|commit| Self::spawn(project, session, commit))
            .map_err(CliFailure::raised::<Setup>)?;
        let unwatched = pushed.iter().find(|commit| dir.status_of(commit).is_none());
        unwatched.map_or(Ok(()), |commit| Err(CliFailure::raised::<Invalid>(format!("no CI watched for {commit}"))))?;
        let clock = WatchClock::start(&settings);
        let settled = std::iter::once(Duration::ZERO)
            .chain(std::iter::repeat(settings.poll_interval().duration()))
            .take_while(|_| clock.remaining(&settings).is_some())
            .find_map(|pause| {
                std::thread::sleep(pause);
                CiStatus::of_all(pushed.iter().map(|commit| dir.status_of(commit)))
            });
        let news = dir.news().map(|news| news.to_string()).unwrap_or_default();
        match settled {
            Some(CiStatus::Passed) => Ok(Output::Listed(Listed::new(format!("{news}{clock}\n")))),
            Some(status) => Err(CliFailure::raised::<Invalid>(format!("{news}\nCI {status}{clock}"))),
            None => Err(CliFailure::raised::<Invalid>(format!("{news}\nCI still running; stopped waiting{clock}"))),
        }
    }

    fn step(
        &self,
        reported: Option<Listed>,
        observed: Result<Observation, Listed>,
    ) -> ControlFlow<Observation, Option<Listed>> {
        let carried = self.news(reported, &observed);
        match observed.ok().filter(|observed| observed.finished().is_some()) {
            Some(observed) => ControlFlow::Break(observed),
            None => ControlFlow::Continue(carried),
        }
    }

    fn news(&self, reported: Option<Listed>, observed: &Result<Observation, Listed>) -> Option<Listed> {
        let overdue = self.settings.run_overdue(self.clock.elapsed());
        let seen = observed
            .as_ref()
            .ok()
            .and_then(Observation::appeared)
            .and_then(|_| self.appeared.set(()).ok())
            .or(self.appeared.get().copied());
        let current = self.dir.status_of(&self.commit);
        let absent = current.filter(|status| *status == CiStatus::Absent);
        let held = current.filter(|status| *status == CiStatus::Failed).unwrap_or(CiStatus::Running);
        let event = match observed {
            Err(trouble) => Some((format!("gh failed: {trouble}"), held)),
            Ok(observed) => observed
                .failed_job()
                .map(|job| (format!("{job} failed; run `meta ci-log`"), CiStatus::Failed))
                .or(overdue.filter(|()| seen.is_none()).map(|()| (Self::NO_RUN_YET.to_owned(), CiStatus::Absent)))
                .or(observed.appeared().and(absent).map(|_| (Self::RUN_APPEARED.to_owned(), CiStatus::Running))),
        };
        match event {
            Some((line, status)) if reported.as_ref().map(ToString::to_string).as_ref() != Some(&line) => {
                self.note(&line, status);
                Some(Listed::new(line))
            }
            Some(_) | None => reported,
        }
    }

    fn conclude(&self, last: Option<&Observation>) -> Output {
        let current = self.dir.status_of(&self.commit);
        let failed = current.filter(|status| *status == CiStatus::Failed);
        let absent = current.filter(|status| *status == CiStatus::Absent);
        let status = failed.or(last.and_then(Observation::finished)).or(absent).unwrap_or(CiStatus::TimedOut);
        let steps: Vec<String> =
            last.iter().flat_map(|observed| observed.failed_steps()).map(ToString::to_string).collect();
        let failed = Some(steps.join(", "))
            .filter(|joined| !joined.is_empty())
            .map(|joined| format!(": {joined}; run `meta ci-log`"));
        let elapsed = Elapsed::new(self.clock.elapsed());
        let detail = status.detail();
        self.note(&format!("{status} after {elapsed}{detail}{}", failed.unwrap_or_default()), status);
        Output::Listed(Listed::new(status))
    }

    fn note(&self, line: &str, status: CiStatus) {
        self.dir.record(format!("{self}: {line}")).ok();
        self.dir.settle(&self.commit, status).ok();
    }
}

impl std::fmt::Display for Watcher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "CI {}{}", self.commit, self.clock)
    }
}
