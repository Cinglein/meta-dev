use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Utc};
use meta_schema::{Id, Task};
use serde::{Deserialize, Serialize};

use crate::request::ci::SessionDir;
use crate::request::scale_policy::Liveness;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Silence {
    task: Id<Task>,
    worktree: PathBuf,
    quiet: Duration,
    limit: Duration,
}

impl Silence {
    pub fn heard(git_dir: &Path, liveness: Liveness, now: DateTime<Utc>) -> Vec<Self> {
        let limit = liveness.silent_after();
        let stamps = limit.map(|_| SessionDir::stamps(git_dir)).unwrap_or_default();
        let live = stamps.into_iter().filter(|stamp| stamp.worktree().path().exists());
        let heard = live.map(|stamp| Self {
            task: stamp.worktree().task(),
            worktree: stamp.worktree().path().to_path_buf(),
            quiet: Duration::from_secs(stamp.quiet(now).as_secs()),
            limit: limit.unwrap_or_default(),
        });
        heard.filter(|silence| silence.quiet > silence.limit).collect()
    }

    pub fn holds(&self, cwd: &Path) -> Option<()> {
        cwd.starts_with(&self.worktree).then_some(())
    }

    pub fn same(&self, other: &Self) -> Option<()> {
        (self.task == other.task).then_some(())
    }
}

impl fmt::Display for Silence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "silent {}: no tool call for {}s, limit {}s",
            self.task,
            self.quiet.as_secs(),
            self.limit.as_secs()
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeDelta, Utc};

    use super::Silence;
    use crate::request::ci::SessionDir;

    const TASK: &str = "task-00000000000000c0";
    const GIT: &str = ".git";
    const NOW: &str = "2026-10-29T12:00:00Z";
    const POLICIES: [&str; 2] = ["silent_after = 1800000", ""];
    const LATER: [TimeDelta; 2] = [TimeDelta::minutes(31), TimeDelta::minutes(30)];
    const SILENCES: [&str; 2] = [
        r#"{"task":"task-00000000000000c0","worktree":"/w/a","quiet":{"secs":9,"nanos":0},"limit":{"secs":1,"nanos":0}}"#,
        r#"{"task":"task-00000000000000c1","worktree":"/w/a","quiet":{"secs":9,"nanos":0},"limit":{"secs":1,"nanos":0}}"#,
    ];
    const HEARD: &str = "silent task-00000000000000c0: no tool call for 1860s, limit 1800s";

    #[test]
    fn a_live_task_worktree_quiet_past_the_policy_limit_is_silent_and_without_a_limit_none_is() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join(TASK).join(GIT)).unwrap();
        let at = NOW.parse::<DateTime<Utc>>().unwrap();
        SessionDir::stamped(&root.path().join(TASK), at).unwrap();
        let git_dir = root.path().join(TASK).join(GIT).canonicalize().unwrap();
        let heard = |policy: &str, later: TimeDelta| -> Vec<String> {
            let found = Silence::heard(&git_dir, toml::from_str(policy).unwrap(), at + later);
            found.iter().map(ToString::to_string).collect()
        };
        let ([limited, unlimited], [past, before]) = (POLICIES, LATER);
        assert_eq!(heard(limited, past), [HEARD]);
        assert!(heard(limited, before).is_empty() && heard(unlimited, past).is_empty());
    }

    #[test]
    fn a_silence_repeats_only_a_silence_of_the_same_task() {
        let [first, other] = SILENCES.map(|text| serde_json::from_str::<Silence>(text).unwrap());
        assert_eq!((first.same(&first), first.same(&other)), (Some(()), None));
    }
}
