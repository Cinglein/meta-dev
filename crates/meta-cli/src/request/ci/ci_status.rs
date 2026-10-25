use std::fmt;
use std::str::FromStr;

use meta_gh::GhRun;

use crate::request::output::Listed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CiStatus {
    Running,
    Passed,
    Failed,
    TimedOut,
}

impl CiStatus {
    pub fn of_runs(runs: &[GhRun]) -> Option<Self> {
        runs.first()
            .filter(|_| runs.iter().all(|run| run.completed().is_some()))
            .map(|_| runs.iter().find(|run| run.succeeded().is_none()).map_or(Self::Passed, |_| Self::Failed))
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

    pub fn failure(self) -> Option<Self> {
        match self {
            Self::Failed => Some(self),
            Self::Running | Self::Passed | Self::TimedOut => None,
        }
    }
}

impl FromStr for CiStatus {
    type Err = std::convert::Infallible;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(match name.trim() {
            "running" => Self::Running,
            "passed" => Self::Passed,
            "failed" => Self::Failed,
            _ => Self::TimedOut,
        })
    }
}

impl fmt::Display for CiStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Running => "running",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::TimedOut => "timed-out",
        })
    }
}
