use meta_gh::{GhRun, Outcome};
use strum::{Display, EnumString};

use crate::request::output::Listed;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum CiStatus {
    Running,
    Passed,
    Failed,
    TimedOut,
}

impl CiStatus {
    pub fn of_runs(runs: &[GhRun]) -> Option<Self> {
        runs.iter().map(GhRun::outcome).collect::<Option<Vec<Outcome>>>().filter(|outcomes| !outcomes.is_empty()).map(
            |outcomes| {
                if outcomes.iter().all(|outcome| *outcome == Outcome::Succeeded) { Self::Passed } else { Self::Failed }
            },
        )
    }

    pub fn stop_reason(self) -> Option<Listed> {
        match self {
            Self::Running => Some(Listed::new(
                "CI for your last push is still running: run `meta ci-wait` (returns at the first failure or at completion)",
            )),
            Self::Failed => Some(Listed::new(
                "CI failed for your last push: fix it, push, then run `meta ci-wait`; or ask the owner",
            )),
            Self::Passed | Self::TimedOut => None,
        }
    }

    pub fn finished(self) -> Option<Self> {
        match self {
            Self::Running => None,
            Self::Passed | Self::Failed | Self::TimedOut => Some(self),
        }
    }
}
