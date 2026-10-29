mod elapsed;

use std::fmt;
use std::time::Duration;

use super::gh::Observation;
use crate::request::ci::ci_status::CiStatus;
use elapsed::Elapsed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    line: String,
    status: CiStatus,
}

impl Note {
    const NO_RUN_YET: &'static str = "no run yet; Stop stays blocked until one appears or the wait is recorded";
    const RUN_APPEARED: &'static str = "run appeared";

    pub fn new(line: impl fmt::Display, status: CiStatus) -> Self {
        Self { line: line.to_string(), status }
    }

    pub fn job_failed(job: impl fmt::Display) -> Self {
        Self::new(format!("{job} failed; run `meta ci-log`"), CiStatus::Failed)
    }

    pub fn awaited(appeared: Option<()>) -> Self {
        appeared.map_or_else(
            || Self::new(Self::NO_RUN_YET, CiStatus::Absent),
            |()| Self::new(Self::RUN_APPEARED, CiStatus::Running),
        )
    }

    pub fn concluded(current: Option<CiStatus>, last: Option<&Observation>, elapsed: Duration) -> Option<Self> {
        let unchecked = current.filter(|status| *status == CiStatus::Unchecked);
        let failed = current.filter(|status| *status == CiStatus::Failed);
        let absent = current.filter(|status| *status == CiStatus::Absent);
        let status = failed.or(last.and_then(Observation::finished)).or(absent).unwrap_or(CiStatus::TimedOut);
        let steps: Vec<String> =
            last.iter().flat_map(|observed| observed.failed_steps()).map(ToString::to_string).collect();
        let failed = Some(steps.join(", "))
            .filter(|joined| !joined.is_empty())
            .map(|joined| format!(": {joined}; run `meta ci-log`"));
        let line = format!("{status} after {}{}{}", Elapsed::new(elapsed), status.detail(), failed.unwrap_or_default());
        unchecked.is_none().then_some(Self { line, status })
    }

    pub fn unchecked(elapsed: Duration, reason: impl fmt::Display) -> Self {
        let unchecked = CiStatus::Unchecked;
        Self::new(format!("{unchecked} after {}: {reason}", Elapsed::new(elapsed)), unchecked)
    }

    pub fn line(&self) -> &str {
        &self.line
    }

    pub fn status(&self) -> CiStatus {
        self.status
    }
}

impl fmt::Display for Note {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.line)
    }
}
