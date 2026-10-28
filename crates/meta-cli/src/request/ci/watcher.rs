mod failure_log;
mod gh;
mod note;
mod poll;
mod reach;
mod remote;
mod session_commits;
mod wait_on_ci;
mod watch_clock;

use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use meta_run::{Budget, EnvPolicy, Run};

use super::ci_status::CiStatus;
use super::commit::Commit;
use super::session_dir::SessionDir;
use super::session_id::SessionId;
use crate::request::cli_failure::{CliFailure, Invalid, Setup};
use crate::request::output::{Listed, Output};
use crate::request::project::{CiSettings, Project};
use crate::request::remote_failure::RemoteFailure;
pub use failure_log::FailureLog;
use gh::{Gh, Observation};
use note::Note;
use poll::Poll;
use reach::Reach;
use remote::Remote;
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
}

impl Watcher {
    const OUT_OF_TIME: &'static str = "couldn't check CI in time; run `meta ci-wait`, then stop again";

    pub fn spawn(project: &Project, session: &SessionId, commit: &Commit) -> Result<(), Listed> {
        let unwatched = |error: std::io::Error| Listed::new(format!("CI not watched: {error}"));
        let dir =
            SessionDir::for_session(project, session).ok_or_else(|| Listed::new("CI not watched: no git directory"))?;
        dir.writable().map_err(unwatched)?;
        let status = dir.status_of(commit).map_or_else(|| dir.watch(commit).map(|()| CiStatus::Running), Ok);
        let idle = status.map_err(unwatched)?.finished().is_none().then(|| dir.claim(commit).map(drop)).flatten();
        idle.map_or(Ok(()), |()| {
            let watch = Run::new(std::env::current_exe().map_err(unwatched)?, EnvPolicy::Inherit);
            let watch = watch.args(["ci-watch", &session.to_string(), &commit.to_string()]).dir(project.root());
            watch.detached().map_err(|failure| Listed::new(format!("CI not watched: {failure}")))
        })
    }

    pub fn watch_all(project: &Project, session: &SessionId, budget: Budget) -> Result<(), Listed> {
        let deadline = project.ci_settings().remote_deadline().min(budget.deadline());
        let seen = SessionCommits::judged(project, session, Reach::Tracking, deadline)?;
        seen.pushed().iter().try_for_each(|commit| Self::spawn(project, session, commit))
    }

    pub fn stop_reason(project: &Project, session: &SessionId) -> Option<Listed> {
        let dir = SessionDir::for_session(project, session)?;
        let deadline = project.ci_settings().stop_limit().deadline();
        let seen = SessionCommits::judged(project, session, Reach::Remote, deadline);
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
        let unresolved = || seen.as_ref().ok().and_then(|seen| WaitOnCi::stop_reason(project, &dir, seen, deadline));
        let reason = unknown.or(unwritable).or(unpushed).or(unwatched).or(known).or_else(unresolved);
        let late = (Instant::now() >= deadline).then(|| Listed::new(Self::OUT_OF_TIME));
        late.or(reason)
    }

    pub fn watch(project: &Project, session: &SessionId, commit: Commit) -> Result<Output, CliFailure> {
        let dir =
            SessionDir::for_session(project, session).ok_or_else(|| CliFailure::raised::<Setup>("no git directory"))?;
        let _lock = dir.claim(&commit).ok_or_else(|| CliFailure::raised::<Invalid>("another watcher has it"))?;
        let settled = dir.status_of(&commit).and_then(CiStatus::finished);
        settled.map_or(Ok(()), |status| Err(CliFailure::raised::<Invalid>(format!("already {status}"))))?;
        let settings = project.ci_settings();
        let clock = WatchClock::start(settings);
        let gh = Gh::new(settings, project.root(), clock.deadline());
        let waits = settings.poll_schedule().waits(dir.expected());
        let remote = Remote::default();
        let watcher = Self { dir, commit, gh, clock, settings: settings.clone() };
        let finished = waits
            .map_while(|wait| watcher.poll(project, &remote, wait))
            .try_fold(Poll::default(), |poll, observed| watcher.step(poll, observed));
        let unchecked = watcher.dir.status_of(&watcher.commit).filter(|status| *status == CiStatus::Unchecked);
        unchecked.map_or_else(
            || {
                let last = finished.break_value();
                let current = watcher.dir.status_of(&watcher.commit);
                let concluded = Note::concluded(current, last.as_ref(), watcher.clock.elapsed());
                watcher.note(&concluded);
                Ok(Output::Listed(Listed::new(concluded.status())))
            },
            |_| Err(CliFailure::raised::<Setup>("remote URL names no GitHub repository")),
        )
    }

    pub fn wait(project: &Project, session: &SessionId) -> Result<Output, CliFailure> {
        let dir =
            SessionDir::for_session(project, session).ok_or_else(|| CliFailure::raised::<Setup>("no git directory"))?;
        let settings = project.ci_settings().clone();
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
            .take_while(|_| clock.window().remaining(&settings).is_some())
            .find_map(|pause| {
                std::thread::sleep(pause);
                CiStatus::of_all(pushed.iter().map(|commit| dir.status_of(commit)))
            });
        let news = dir.news().map(|news| news.to_string()).unwrap_or_default();
        match settled {
            Some(CiStatus::Passed) => Ok(Output::Listed(Listed::new(format!("{news}\n")))),
            Some(status) => Err(CliFailure::raised::<Invalid>(format!("{news}\nCI {status}"))),
            None => Err(CliFailure::raised::<Invalid>(format!("{news}\nCI still running; stopped waiting"))),
        }
    }

    fn poll(&self, project: &Project, remote: &Remote, wait: Duration) -> Option<Result<Observation, Listed>> {
        let resolved = remote.resolved(project).map_err(RemoteFailure::retry);
        let refused = resolved.as_ref().err().and_then(|retry| retry.as_ref().err());
        let stopped = refused.map(|reason| Note::unchecked(self.clock.elapsed(), reason));
        stopped.map_or(Some(()), |note| {
            self.note(&note);
            None
        })?;
        self.clock.pause(wait, &self.settings)?;
        let repository = resolved.map_err(|retry| retry.unwrap_or_else(std::convert::identity));
        let observed = repository.and_then(|repository| self.gh.observe(repository, &self.commit));
        let learned = observed.as_ref().ok().and_then(Observation::expected);
        learned.map(|expected| self.dir.learn(expected).ok());
        Some(observed)
    }

    fn step(&self, poll: Poll, observed: Result<Observation, Listed>) -> ControlFlow<Observation, Poll> {
        let overdue = self.settings.run_overdue(self.clock.elapsed());
        let poll = poll.step(&observed, self.dir.status_of(&self.commit), overdue);
        poll.fresh().and_then(|note| self.note(note));
        match observed.ok().filter(|observed| observed.finished().is_some()) {
            Some(observed) => ControlFlow::Break(observed),
            None => ControlFlow::Continue(poll),
        }
    }

    fn note(&self, note: &Note) -> Option<()> {
        self.dir.record(format!("{self}: {note}")).ok();
        self.dir.settle(&self.commit, note.status()).ok()
    }
}

impl std::fmt::Display for Watcher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "CI {}", self.commit)
    }
}
