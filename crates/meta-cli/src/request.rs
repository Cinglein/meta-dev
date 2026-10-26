mod ci;
mod cli_failure;
mod hook;
mod live_github;
mod local;
mod output;
mod parsed;
mod project;
mod status_view;
mod timed_command;
mod writing;

use meta_listing::{Kind, Scope};
use meta_log::LogStore;
use meta_schema::{Actor, Block, Claim, EntryNote, Event, NewConcern, NewIssue, NewTask, TaskRef};

use ci::{Commit, FailureLog, SessionId, Watcher};
use cli_failure::{CliFailure, Invalid, Misuse, Setup};
use hook::HookEvent;
use live_github::LiveGithub;
use local::{LocalClone, Sweep};
use output::{Listed, Output};
use parsed::{OptionName, Parsed};
use project::Project;
use status_view::StatusView;
use writing::Writing;

#[derive(Debug)]
pub enum Request {
    Write(Event),
    ClaimWorktree(Claim),
    Worktrees,
    Gc(Sweep),
    List(Option<Kind>, Scope),
    Status(StatusView),
    Hook(HookEvent),
    WatchCi(SessionId, Commit),
    WaitCi(SessionId),
    CiLog(SessionId),
}

impl TryFrom<Vec<String>> for Request {
    type Error = CliFailure;

    fn try_from(args: Vec<String>) -> Result<Self, Self::Error> {
        let (_program, words) =
            args.split_first().ok_or_else(|| CliFailure::raised::<Misuse>("missing program name"))?;
        let (name, rest) = words.split_first().ok_or_else(|| CliFailure::raised::<Misuse>("missing command"))?;
        let parsed = Parsed::try_from(rest)?;
        let task = || parsed.task().map(TaskRef::new);
        match name.as_str() {
            "concern" => {
                let parsed = parsed.allowing(&[OptionName::Source, OptionName::Tag, OptionName::Division])?;
                Ok(Self::Write(Event::ConcernIdentified(NewConcern::new(
                    parsed.division()?,
                    parsed.text()?,
                    parsed.source(),
                    parsed.tags(),
                ))))
            }
            "issue" => {
                let parsed = parsed.allowing(&[OptionName::Division])?;
                Ok(Self::Write(Event::IssueReported(NewIssue::new(parsed.division()?, parsed.text()?))))
            }
            "open" => {
                let parsed = parsed.allowing(&[
                    OptionName::Addresses,
                    OptionName::Division,
                    OptionName::Effort,
                    OptionName::Due,
                ])?;
                let opened = NewTask::new(parsed.division()?, parsed.text()?, parsed.addresses());
                let sized = parsed.effort().into_iter().fold(opened, NewTask::allowing);
                Ok(Self::Write(Event::TaskOpened(parsed.due().into_iter().fold(sized, NewTask::due_by))))
            }
            "wait" => {
                let parsed = parsed.allowing(&[OptionName::On])?;
                Ok(Self::Write(Event::TaskBlocked(Block::new(parsed.task()?, parsed.blocker()?))))
            }
            "resume" => Ok(Self::Write(Event::TaskUnblocked(task()?))),
            "link" => Ok(Self::Write(Event::TaskLinked(parsed.allowing(&[])?.task_link()?))),
            "claim" => {
                let parsed = parsed.allowing(&[OptionName::From, OptionName::Worktree])?;
                let claim = Claim::new(parsed.task()?, parsed.previous_claimant());
                Ok(match parsed.flagged(OptionName::Worktree) {
                    Some(()) => Self::ClaimWorktree(claim),
                    None => Self::Write(Event::TaskClaimed(claim)),
                })
            }
            "worktrees" => parsed.allowing(&[]).map(|_| Self::Worktrees),
            "gc" => {
                let planned = parsed.allowing(&[OptionName::DryRun])?.flagged(OptionName::DryRun);
                Ok(Self::Gc(planned.map_or(Sweep::Done, |()| Sweep::Planned)))
            }
            "release" => Ok(Self::Write(Event::TaskReleased(task()?))),
            "done" => Ok(Self::Write(Event::TaskDone(task()?))),
            "reopen" => Ok(Self::Write(Event::TaskReopened(task()?))),
            "retire" => parsed
                .allowing(&[])?
                .entry_and_text(|entry, reason| Self::Write(Event::EntryRetired(EntryNote::new(entry, reason)))),
            "correct" => parsed
                .allowing(&[])?
                .entry_and_text(|entry, text| Self::Write(Event::EntryCorrected(EntryNote::new(entry, text)))),
            "list" => Ok(Self::List(parsed.kind()?, parsed.allowing(&[OptionName::All])?.scope())),
            "status" => Ok(Self::Status(parsed.allowing(&[OptionName::Flags])?.view())),
            "hook" => Ok(Self::Hook(parsed.allowing(&[])?.text()?.to_string().parse()?)),
            "ci-watch" => parsed.allowing(&[])?.session_and_commit(Self::WatchCi),
            "ci-wait" => Self::session().map(Self::WaitCi),
            "ci-log" => Self::session().map(Self::CiLog),
            other => Err(CliFailure::raised::<Misuse>(format!("unknown command `{other}`"))),
        }
    }
}

impl Request {
    pub fn answer(self) -> Result<Output, CliFailure> {
        match self {
            Self::Hook(event) => event.respond(),
            other => other.answer_in(&Project::try_from(
                std::env::current_dir().map_err(CliFailure::raised::<Setup>)?.as_path(),
            )?),
        }
    }

    pub fn answer_in(self, project: &Project) -> Result<Output, CliFailure> {
        match self {
            Self::Hook(event) => event.respond(),
            Self::WatchCi(session, commit) => Watcher::watch(project, &session, commit),
            Self::WaitCi(session) => Watcher::wait(project, &session),
            Self::CiLog(session) => FailureLog::excerpt(project, &session),
            Self::Status(view) => view.answer(&project.log()?, |query| LiveGithub::for_project(project).read(query)),
            Self::ClaimWorktree(claim) => LocalClone::try_from(project)?.claimed(project, claim, Self::actor()?),
            Self::Worktrees => LocalClone::try_from(project)?.worktrees(project),
            Self::Gc(sweep) => LocalClone::try_from(project)?.collected(project, sweep),
            logged => logged.answer_from(&project.log()?, Self::actor),
        }
    }

    fn answer_from(
        self,
        log: &impl LogStore,
        actor: impl FnOnce() -> Result<Actor, CliFailure>,
    ) -> Result<Output, CliFailure> {
        match self {
            Self::Write(event) => Writing::new(log, actor()?, event).entered(),
            Self::List(kind, scope) => Listed::from_log(log, kind, scope).map(Output::Listed),
            Self::Hook(_)
            | Self::WatchCi(..)
            | Self::WaitCi(_)
            | Self::CiLog(_)
            | Self::Status(_)
            | Self::ClaimWorktree(_)
            | Self::Worktrees
            | Self::Gc(_) => Err(CliFailure::raised::<Misuse>("not a meta/log request")),
        }
    }

    fn session() -> Result<SessionId, CliFailure> {
        std::env::var(SessionId::VARIABLE)
            .map_err(|_| {
                CliFailure::raised::<Setup>(format!(
                    "{} is not set; run inside a Claude session with the plugin",
                    SessionId::VARIABLE
                ))
            })
            .and_then(|session| session.parse().map_err(CliFailure::raised::<Invalid>))
    }

    fn actor() -> Result<Actor, CliFailure> {
        std::env::var("META_ACTOR")
            .ok()
            .filter(|name| !name.is_empty())
            .map_or_else(|| Ok(Actor::owner()), |name| name.parse().map_err(CliFailure::raised::<Invalid>))
    }
}

#[cfg(test)]
mod tests;
