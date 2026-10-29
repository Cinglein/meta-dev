mod hush;

use std::fmt;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use meta_schema::{Block, Id, Task};
use meta_state::{State, TaskStatus};
use serde::{Deserialize, Serialize};

use super::fault::Fault;
use crate::request::ci::SessionDir;
use crate::request::cli_failure::CliFailure;
use crate::request::hook::CallPhase;
use crate::request::local::local_trail::LocalTrail;
use crate::request::local::recorded_worktree::RecordedWorktree;
use crate::request::scale_policy::Liveness;
use hush::Hush;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Silence {
    task: Id<Task>,
    worktree: PathBuf,
    hush: Hush,
}

impl Silence {
    pub fn heard(
        git_dir: &Path,
        liveness: Liveness,
        state: impl FnOnce() -> Result<State, CliFailure>,
        now: DateTime<Utc>,
    ) -> Result<Vec<Self>, CliFailure> {
        let recorded = liveness.limit(CallPhase::Finished).map(|_| LocalTrail::read(git_dir)).transpose()?;
        let live: Vec<RecordedWorktree> = recorded
            .map(|trail| trail.current())
            .unwrap_or_default()
            .into_iter()
            .filter(|tree| tree.path().exists())
            .collect();
        let watched = live.first().map(|_| state().map(|state| Self::watched(&state))).transpose()?.unwrap_or_default();
        let kept = live.into_iter().filter(|tree| watched.contains(&tree.task()));
        Ok(kept.filter_map(|tree| Self::judged(git_dir, liveness, &tree, now)).collect())
    }

    fn watched(state: &State) -> Vec<Id<Task>> {
        let active = state
            .tasks()
            .iter()
            .filter(|task| !matches!(state.status_of(task), TaskStatus::Done(_) | TaskStatus::Retired));
        let unasked = active.filter(|task| task.progress().waiting().and_then(Block::reply_due).is_none());
        unasked.map(|task| task.new_task().task()).collect()
    }

    fn judged(git_dir: &Path, liveness: Liveness, tree: &RecordedWorktree, now: DateTime<Utc>) -> Option<Self> {
        let hush = match SessionDir::stamp_of(git_dir, tree.task()) {
            Err(failure) => Some(Hush::Unreadable(Fault::shown(failure))),
            Ok(None) => Some(Hush::Unstamped),
            Ok(Some(stamp)) => Hush::timed(stamp.call(), stamp.quiet(now), liveness.limit(stamp.call())?),
        }?;
        let worktree = std::fs::canonicalize(tree.path()).unwrap_or_else(|_| tree.path().to_path_buf());
        Some(Self { task: tree.task(), worktree, hush })
    }

    pub fn holds(&self, cwd: &Path) -> Option<()> {
        self.hush.stops().filter(|()| cwd.starts_with(&self.worktree))
    }

    pub fn same(&self, other: &Self) -> Option<()> {
        (self.task == other.task && self.hush.label() == other.hush.label()).then_some(())
    }
}

impl fmt::Display for Silence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}: {}", self.hush.label(), self.task, self.hush)
    }
}

#[cfg(test)]
mod tests;
