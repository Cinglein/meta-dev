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
    Unchecked,
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
            Self::Running | Self::Passed | Self::TimedOut | Self::Unavailable | Self::Absent | Self::Unchecked => self,
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Self::Unavailable => ": jobs never started; push a new commit to retry, then run `meta ci-wait`",
            Self::Absent => ": no run for this commit; no CI for this push",
            Self::Running | Self::Passed | Self::Failed | Self::TimedOut | Self::Unchecked => "",
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
            Self::Passed | Self::TimedOut | Self::Unavailable | Self::Absent | Self::Unchecked => None,
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
            Self::Unchecked => Some(
                "couldn't check CI for your last push: fix the remote URL, push a new commit, then run `meta ci-wait`",
            ),
            Self::Running | Self::Passed | Self::Failed => None,
        }
    }

    pub fn finished(self) -> Option<Self> {
        match self {
            Self::Running => None,
            Self::Passed | Self::Failed | Self::TimedOut | Self::Unavailable | Self::Absent | Self::Unchecked => {
                Some(self)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CiStatus::{self, Absent, Failed, Passed, Running, TimedOut, Unavailable, Unchecked};

    const EVERY: [CiStatus; 7] = [Running, Passed, Failed, TimedOut, Unavailable, Absent, Unchecked];
    const TOLD: [usize; 7] = [1, 0, 1, 1, 1, 1, 1];
    const PUSH: &str = "your last push";

    #[test]
    fn a_failure_decides_several_runs_else_any_running_waits_else_the_first_unpassed_shows_and_each_unpassed_is_told_once()
     {
        let of = |statuses: &[Option<CiStatus>]| CiStatus::of_all(statuses.iter().copied());
        let decided = [of(&[Some(Passed), Some(Failed), None]), of(&[Some(Passed), Some(Running)])];
        let settled = [of(&[Some(Passed), Some(TimedOut), Some(Absent)]), of(&[Some(Passed), Some(Passed)])];
        assert_eq!([decided, settled], [[Some(Failed), None], [Some(TimedOut), Some(Passed)]]);
        let told = EVERY.map(|status| {
            let stopped = status.stop_reason().map(|reason| reason.to_string());
            let texts = stopped.into_iter().chain(status.unresolved().map(str::to_owned));
            texts.filter(|text| text.contains(PUSH)).count()
        });
        assert_eq!(told, TOLD);
    }
}
