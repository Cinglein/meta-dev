use std::fmt;
use std::time::Instant;

use meta_schema::{CommitId, ExternalKind, Id, Task};
use meta_state::{State, TaskStatus};

use super::session_commits::SessionCommits;
use crate::request::ci::ci_status::CiStatus;
use crate::request::ci::{Commit, SessionDir};
use crate::request::cli_failure::CliFailure;
use crate::request::output::Listed;
use crate::request::project::{BranchName, CiSettings, Project};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitOnCi {
    Recorded,
    NotHead(CommitId),
    Unrecorded(Id<Task>),
    Waiting(Id<Task>, TaskStatus),
    Done(Id<Task>),
    Taskless,
}

impl WaitOnCi {
    const ON_CI: TaskStatus = TaskStatus::WaitingOnExternal(ExternalKind::Ci);
    const READY: [TaskStatus; 2] = [TaskStatus::Open, TaskStatus::Assigned];

    pub fn stop_reason(
        project: &Project,
        dir: &SessionDir,
        seen: &SessionCommits,
        deadline: Instant,
    ) -> Option<Listed> {
        let commits = seen.pushed();
        let unresolved = |commit: &Commit| dir.status_of(commit).and_then(CiStatus::unresolved);
        commits.iter().find_map(unresolved)?;
        let budget = project.ci_settings().budget(CiSettings::ROUTINE).within(deadline);
        let read = project.log(budget).and_then(|log| log.read_listed(seen.heads()).map_err(CliFailure::from));
        let state = read.map(|read| State::fold(read.records()));
        let head = seen.checkout().head().and_then(Commit::commit_id);
        let branch = seen.checkout().branch().filter(|branch| project.config().protects(branch).is_none());
        let task = branch.and_then(BranchName::task);
        commits.iter().find_map(|commit| {
            let what = unresolved(commit)?;
            Self::judging(&state, commit, head.as_ref(), task).map_or_else(Some, |wait| wait.reason(what))
        })
    }

    fn judging(
        state: &Result<State, CliFailure>,
        commit: &Commit,
        head: Option<&CommitId>,
        task: Option<Id<Task>>,
    ) -> Result<Self, Listed> {
        state
            .as_ref()
            .map(|state| Self::in_state(state, commit.commit_id().as_ref(), head, task))
            .map_err(|failure| Listed::new(format!("meta/log unreadable: {failure}; fix it or ask the owner")))
    }

    fn in_state(state: &State, commit: Option<&CommitId>, head: Option<&CommitId>, task: Option<Id<Task>>) -> Self {
        let covered = commit.and_then(|commit| {
            state
                .tasks()
                .iter()
                .find(|known| state.status_of(known) == Self::ON_CI && known.progress().waited_at() == Some(commit))
        });
        let elsewhere = commit.filter(|commit| Some(*commit) != head).map(|commit| Self::NotHead(commit.clone()));
        let advice = || task.map_or(Self::Taskless, |task| Self::of_task(state, task));
        covered.map(|_| Self::Recorded).or(elsewhere).unwrap_or_else(advice)
    }

    fn of_task(state: &State, task: Id<Task>) -> Self {
        let known = state.tasks().iter().find(|known| known.new_task().task() == task);
        known.map_or(Self::Taskless, |known| {
            let status = state.status_of(known);
            let retired = (status == TaskStatus::Retired).then_some(Self::Taskless);
            let ready = Self::READY.contains(&status).then_some(Self::Unrecorded(task));
            let done = known.progress().done().map(|_| Self::Done(task));
            retired.or(ready).or(done).unwrap_or(Self::Waiting(task, status))
        })
    }

    fn reason(self, unresolved: &str) -> Option<Listed> {
        match self {
            Self::Recorded => None,
            Self::NotHead(_) | Self::Unrecorded(_) | Self::Waiting(..) | Self::Done(_) | Self::Taskless => {
                Some(Listed::new(unresolved).then(self))
            }
        }
    }
}

impl fmt::Display for WaitOnCi {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recorded => Ok(()),
            Self::NotHead(commit) => write!(
                formatter,
                " ({commit}, not HEAD); a task waits at one commit only; each other unresolved commit needs its own task; or run `git checkout --detach {commit}`, then `meta wait <task> --on ci` there; push a rerun to that commit's branch; check the branch out again after detaching; a detached HEAD cannot push"
            ),
            Self::Unrecorded(task) => write!(
                formatter,
                "; or run `meta wait {task} --on ci` here (`meta claim {task}` first if you do not hold it)"
            ),
            Self::Waiting(task, status) => write!(
                formatter,
                "; or task {task} is {status}: run `meta resume {task}` first, then `meta wait {task} --on ci` here"
            ),
            Self::Done(task) => write!(
                formatter,
                "; or task {task} is done: run `meta reopen {task}` and `meta claim {task}`, or open a new task, then `meta wait <task> --on ci` here"
            ),
            Self::Taskless => formatter.write_str(
                "; or open or claim a task (`meta open <text> --division <path>`, `meta claim <task>`), then run `meta wait <task> --on ci` here",
            ),
        }
    }
}

#[cfg(test)]
mod tests;
