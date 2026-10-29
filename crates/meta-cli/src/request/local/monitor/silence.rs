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
        watched: &[Id<Task>],
        now: DateTime<Utc>,
    ) -> Result<Vec<Self>, CliFailure> {
        liveness.limit(CallPhase::Finished).map_or_else(
            || Ok(Vec::new()),
            |_| {
                let recorded = LocalTrail::read(git_dir)?.current();
                let live = recorded.into_iter().filter(|tree| watched.contains(&tree.task()) && tree.path().exists());
                Ok(live.filter_map(|tree| Self::judged(git_dir, liveness, &tree, now)).collect())
            },
        )
    }

    pub fn watched(state: &State) -> Vec<Id<Task>> {
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
mod tests {
    use chrono::{DateTime, TimeDelta, Utc};

    use super::{Hush, Silence};
    use crate::request::ci::SessionDir;
    use crate::request::hook::CallPhase;
    use crate::request::local::local_trail::LocalTrail;
    use crate::request::local::recorded_worktree::RecordedWorktree;
    use crate::request::local::trail_entry::TrailEntry;

    const TASKS: [&str; 3] = ["task-00000000000000c0", "task-00000000000000c1", "task-00000000000000c2"];
    const CALLS: [Option<CallPhase>; 3] = [Some(CallPhase::Finished), Some(CallPhase::Started), None];
    const GIT: &str = ".git";
    const LINK: &str = "gitdir: ../.git\n";
    const NOW: &str = "2026-10-29T12:00:00Z";
    const POLICIES: [&str; 2] = ["silent_after = 1800000\nlong_call_after = 3600000", ""];
    const LATER: TimeDelta = TimeDelta::minutes(31);
    const HEARD: [&str; 2] = [
        "silent task-00000000000000c0: no tool call for 1860s, limit 1800s",
        "unstamped task-00000000000000c2: live worktree with no tool-call stamp",
    ];
    const SILENCES: [&str; 2] = [
        r#"{"task":"task-00000000000000c0","worktree":"/w/a","hush":"Unstamped"}"#,
        r#"{"task":"task-00000000000000c1","worktree":"/w/a","hush":"Unstamped"}"#,
    ];

    #[test]
    fn watched_worktrees_quiet_past_the_limit_or_unstamped_are_heard_and_a_call_in_progress_is_not_silence() {
        let root = tempfile::tempdir().unwrap();
        let git_dir = root.path().join(GIT);
        std::fs::create_dir(&git_dir).unwrap();
        let at = NOW.parse::<DateTime<Utc>>().unwrap();
        TASKS.iter().zip(CALLS).for_each(|(task, call)| {
            let worktree = root.path().join(task);
            std::fs::create_dir(&worktree).and_then(|()| std::fs::write(worktree.join(GIT), LINK)).unwrap();
            let created = TrailEntry::Created(RecordedWorktree::new(worktree.clone(), task.parse().unwrap()));
            LocalTrail::append(&git_dir, &created).unwrap();
            call.and_then(|call| SessionDir::stamped(&worktree, call, at)).transpose().unwrap();
        });
        let watched = TASKS.map(|task| task.parse().unwrap());
        let heard = |policy: &str| -> Vec<String> {
            let found =
                Silence::heard(&git_dir.canonicalize().unwrap(), toml::from_str(policy).unwrap(), &watched, at + LATER);
            found.unwrap().iter().map(ToString::to_string).collect()
        };
        let [limited, unlimited] = POLICIES;
        assert_eq!(heard(limited), HEARD);
        assert_eq!(heard(unlimited), Vec::<String>::new());
    }

    #[test]
    fn a_silence_repeats_only_one_of_the_same_task_and_a_quiet_spell_at_the_limit_is_not_silent() {
        let [first, other] = SILENCES.map(|text| serde_json::from_str::<Silence>(text).unwrap());
        assert_eq!((first.same(&first), first.same(&other)), (Some(()), None));
        assert_eq!(Hush::timed(CallPhase::Finished, LATER.to_std().unwrap(), LATER.to_std().unwrap()), None);
    }
}
