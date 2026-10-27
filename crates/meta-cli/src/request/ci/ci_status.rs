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
    Unavailable,
    Absent,
}

impl CiStatus {
    pub fn of_runs(runs: &[GhRun]) -> Option<Self> {
        let outcomes: Option<Vec<Outcome>> = runs.iter().map(GhRun::outcome).collect();
        runs.first().map(|_| outcomes.as_deref().map_or(Self::Running, Self::of_outcomes))
    }

    fn of_outcomes(outcomes: &[Outcome]) -> Self {
        if outcomes.iter().all(|outcome| *outcome == Outcome::Succeeded) { Self::Passed } else { Self::Failed }
    }

    pub fn of_all(statuses: impl Iterator<Item = Option<Self>>) -> Option<Self> {
        let finished: Vec<Option<Self>> = statuses.map(|status| status.and_then(Self::finished)).collect();
        let failed = finished.contains(&Some(Self::Failed)).then_some(Self::Failed);
        let all: Option<Vec<Self>> = finished.into_iter().collect();
        failed.or(all.map(|all| all.into_iter().find(|status| *status != Self::Passed).unwrap_or(Self::Passed)))
    }

    pub fn unstarted(self) -> Self {
        match self {
            Self::Failed => Self::Unavailable,
            Self::Running | Self::Passed | Self::TimedOut | Self::Unavailable | Self::Absent => self,
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Self::Unavailable => ": jobs never started; push a new commit to retry, then run `meta ci-wait`",
            Self::Absent => ": no run for this commit; no CI for this push",
            Self::Running | Self::Passed | Self::Failed | Self::TimedOut => "",
        }
    }

    pub fn stop_reason(self) -> Option<Listed> {
        match self {
            Self::Running => Some(Listed::new(
                "CI for your last push is still running: run `meta ci-wait` (returns at the first failure or at completion)",
            )),
            Self::Failed => Some(Listed::new(
                "CI failed for your last push: fix it, push, then run `meta ci-wait`; or ask the owner",
            )),
            Self::Passed | Self::TimedOut | Self::Unavailable | Self::Absent => None,
        }
    }

    pub fn unresolved(self) -> Option<&'static str> {
        match self {
            Self::TimedOut => {
                Some("CI for your last push timed out: push a new commit to rerun it, then run `meta ci-wait`")
            }
            Self::Unavailable => {
                Some("CI jobs for your last push never started: push a new commit to retry, then run `meta ci-wait`")
            }
            Self::Absent => {
                Some("no CI run for your last push: push a new commit to start one, then run `meta ci-wait`")
            }
            Self::Running | Self::Passed | Self::Failed => None,
        }
    }

    pub fn finished(self) -> Option<Self> {
        match self {
            Self::Running => None,
            Self::Passed | Self::Failed | Self::TimedOut | Self::Unavailable | Self::Absent => Some(self),
        }
    }
}
