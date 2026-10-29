mod checkout;
mod ci;
mod cli_failure;
mod git_link;
mod heartbeat;
mod hook;
mod live_github;
mod local;
mod merge_queue;
mod output;
mod parsed;
mod project;
mod rate;
mod relocate;
mod remote_failure;
mod repository;
mod scale_policy;
mod status_view;
mod trace_view;
mod writing;

use std::path::PathBuf;

use chrono::Utc;
use meta_listing::{Kind, Scope};
use meta_log::LogStore;
use meta_schema::{Actor, Block, Claim, EntryNote, Event, NewConcern, NewIssue, NewTask, TaskRef};

use ci::{Commit, FailureLog, SessionId, Watcher};
use cli_failure::{CliFailure, Invalid, Misuse, Setup};
use heartbeat::Heartbeat;
use hook::HookEvent;
use live_github::LiveGithub;
pub use local::TraceRecord;
use local::{LocalClone, Monitor, Sweep};
use merge_queue::MergeQueue;
use output::{Listed, Output};
use parsed::{OptionName, Parsed};
use project::{CiSettings, Project};
use rate::Rate;
use relocate::Relocate;
use scale_policy::ScalePolicy;
use status_view::StatusView;
use trace_view::TraceView;
use writing::Writing;

#[derive(Debug)]
pub enum Request {
    Write(Event),
    Wait(Block),
    Rate(Rate),
    Move(Relocate),
    ClaimWorktree(Claim),
    Worktrees,
    Gc(Sweep),
    GcRestore(PathBuf),
    List(Option<Kind>, Scope),
    Status(StatusView),
    Hook(HookEvent),
    Heartbeat(Heartbeat),
    Monitor(Monitor),
    WatchCi(SessionId, Commit),
    WaitCi(SessionId),
    CiLog(SessionId),
    Trace(TraceView),
    Queue(MergeQueue),
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
                let parsed = parsed.allowing(&[
                    OptionName::Source,
                    OptionName::Tag,
                    OptionName::Division,
                    OptionName::Severity,
                ])?;
                let concern = NewConcern::new(parsed.division()?, parsed.text()?, parsed.source(), parsed.tags());
                Ok(Self::Write(Event::ConcernIdentified(
                    parsed.severity().into_iter().fold(concern, NewConcern::rated),
                )))
            }
            "issue" => {
                let parsed = parsed.allowing(&[OptionName::Division, OptionName::Severity])?;
                let issue = NewIssue::new(parsed.division()?, parsed.text()?);
                Ok(Self::Write(Event::IssueReported(parsed.severity().into_iter().fold(issue, NewIssue::rated))))
            }
            "rate" => Ok(Self::Rate(parsed.allowing(&[OptionName::Scenario])?.rating()?)),
            "move" => {
                let parsed = parsed.allowing(&[OptionName::Division, OptionName::Severity, OptionName::Scenario])?;
                Ok(Self::Move(parsed.relocation()?))
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
                let parsed = parsed.allowing(&[OptionName::On, OptionName::Due])?;
                let block = Block::new(parsed.task()?, parsed.blocker()?);
                Ok(Self::Wait(parsed.due().into_iter().fold(block, Block::reply_by)))
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
                let parsed = parsed.allowing(&[OptionName::DryRun])?;
                let planned = parsed.flagged(OptionName::DryRun).map_or(Sweep::Done, |()| Sweep::Planned);
                Ok(parsed.restored().map_or(Self::Gc(planned), Self::GcRestore))
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
            "heartbeat" => Ok(Self::Heartbeat(parsed.allowing(&[])?.text()?.to_string().parse()?)),
            "monitor" => Ok(Self::Monitor(parsed.allowing(&[])?.text()?.to_string().parse()?)),
            "ci-watch" => parsed.allowing(&[])?.session_and_commit(Self::WatchCi),
            "ci-wait" => Self::session().map(Self::WaitCi),
            "ci-log" => Self::session().map(Self::CiLog),
            "trace" => Ok(Self::Trace(parsed.allowing(&[])?.text()?.to_string().parse()?)),
            "queue" => parsed.allowing(&[])?.queue().map(Self::Queue),
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
            Self::Heartbeat(command) => command.answer(project),
            Self::WatchCi(session, commit) => Watcher::watch(project, &session, commit),
            Self::WaitCi(session) => Watcher::wait(project, &session),
            Self::CiLog(session) => FailureLog::excerpt(project, &session),
            Self::Trace(view) => TraceRecord::shown(&LocalClone::try_from(project)?, view),
            Self::Queue(queue) => queue.run(project),
            Self::Status(view) => {
                let local = match view {
                    StatusView::Flags => Monitor::recent(&LocalClone::try_from(project)?, Utc::now())?,
                    StatusView::Tree => None,
                };
                let answered = view.answer(
                    &project.log(project.ci_settings().budget(CiSettings::ROUTINE))?,
                    ScalePolicy::read(project.root())?.scales(),
                    |query| LiveGithub::for_project(project).read(query),
                )?;
                Ok(answered.raised_with(local))
            }
            Self::Monitor(command) => command.answer(
                project,
                &LocalClone::try_from(project)?,
                &project::HeartbeatSettings::try_from(project.config())?,
            ),
            Self::ClaimWorktree(claim) => LocalClone::try_from(project)?.claimed(project, claim, Self::actor()?),
            Self::Worktrees => LocalClone::try_from(project)?.worktrees(project),
            Self::Gc(sweep) => LocalClone::try_from(project)?.collected(project, sweep),
            Self::GcRestore(quarantine) => LocalClone::try_from(project)?.unstranded(&quarantine),
            logged @ (Self::List(..) | Self::Write(_) | Self::Rate(_) | Self::Move(_)) => logged.answer_from(
                &project.log(project.ci_settings().budget(CiSettings::ROUTINE))?,
                &ScalePolicy::read(project.root())?,
                Self::actor,
            ),
            Self::Wait(block) => {
                let head = project.head().and_then(|head| head.commit_id());
                let block = head.into_iter().fold(block, Block::at_head);
                let asked = ScalePolicy::read(project.root())?.liveness().asked(block, Utc::now());
                Self::Write(Event::TaskBlocked(asked)).answer_in(project)
            }
        }
    }

    fn answer_from(
        self,
        log: &impl LogStore,
        policy: &ScalePolicy,
        actor: impl FnOnce() -> Result<Actor, CliFailure>,
    ) -> Result<Output, CliFailure> {
        match self {
            Self::Write(event) => {
                policy.scales().admits(&event).map_err(CliFailure::raised::<Invalid>)?;
                Writing::new(log, actor()?, event).entered()
            }
            Self::Rate(rate) => rate.answer(log, policy.scales(), actor()?),
            Self::Move(relocate) => relocate.answer(log, policy, actor()?),
            Self::List(kind, scope) => Listed::from_log(log, kind, scope).map(Output::Listed),
            Self::Wait(block) => Self::Write(Event::TaskBlocked(block)).answer_from(log, policy, actor),
            Self::Hook(_)
            | Self::Heartbeat(_)
            | Self::Monitor(_)
            | Self::WatchCi(..)
            | Self::WaitCi(_)
            | Self::CiLog(_)
            | Self::Trace(_)
            | Self::Queue(_)
            | Self::Status(_)
            | Self::ClaimWorktree(_)
            | Self::Worktrees
            | Self::Gc(_)
            | Self::GcRestore(_) => Err(CliFailure::raised::<Misuse>("not a meta/log request")),
        }
    }

    fn session() -> Result<SessionId, CliFailure> {
        let directory = std::env::current_dir().map_err(CliFailure::raised::<Setup>)?;
        SessionId::chosen(std::env::var(SessionId::VARIABLE).ok(), &directory).ok_or_else(|| {
            CliFailure::raised::<Setup>(format!(
                "{} is not set and this directory is not a claimed task worktree; run `meta claim <task> --worktree` and work in its path, or set {}",
                SessionId::VARIABLE,
                SessionId::VARIABLE
            ))
        })
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
