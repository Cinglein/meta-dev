mod step_failures;

use std::io;
use std::path::PathBuf;

use meta_golden::GoldenFailure;
use meta_log::LogFailure;
use meta_run::Unread;
use meta_severity::{Demotions, ScaleFailure, Strands};
use task::{CheckName, FailureSet, Unattributed};
use thiserror::Error;

use crate::task::adoption::Gaps;
use crate::task::ci::{Busy, OutsideConfig, Unready, Untracked};
use crate::task::ratchet::{Raised, Ratchet};
use crate::task::revision::ForeignEmails;
use crate::task::test_processes::Unmatched;
use crate::task::{Misplaced, TrackedBuildFiles};
pub use step_failures::{StepFailure, StepFailures};

impl Failure {
    pub fn found(&self, job: &CheckName) -> FailureSet {
        match self {
            Failure::Task(failure) => failure.found(job),
            Failure::Steps(steps) => steps.found(),
            Failure::Unreached { failure, .. } => failure.found(job),
            _ => FailureSet::default(),
        }
    }
}

#[derive(Debug, Error)]
pub enum Failure {
    #[error("{0}")]
    Task(#[from] task::Failure),
    #[error("{0}")]
    Steps(Box<StepFailures>),
    #[error("{0}")]
    Ratchet(Ratchet),
    #[error("{0}")]
    Raised(Raised),
    #[error("{0}")]
    Unmatched(Unmatched),
    #[error("baseline: {0}")]
    Baseline(#[from] toml::de::Error),
    #[error("baseline: {0}")]
    Written(#[from] toml::ser::Error),
    #[error("step summary: {0}")]
    Summary(io::Error),
    #[error("{0}")]
    Unattributed(#[from] Unattributed),
    #[error("failures file: {0}")]
    Failures(io::Error),
    #[error("gate not posted: {0}")]
    Unready(#[from] Unready),
    #[error("{0}")]
    Busy(Busy),
    #[error("{0}")]
    Configured(OutsideConfig),
    #[error("cargo metadata: {0}")]
    Metadata(#[from] serde_json::Error),
    #[error("merge-base [spec] in tooling/policy.toml: {0}")]
    PriorSpec(toml::de::Error),
    #[error("merge-base spec file {} is unreadable", .0.display())]
    PriorSpecFile(PathBuf),
    #[error("{0}")]
    Identity(ForeignEmails),
    #[error("{0}")]
    Stranded(Strands),
    #[error("{0}")]
    Demoted(Demotions),
    #[error("merge-base tooling/policy.toml does not parse: {0}")]
    PriorPolicy(toml::de::Error),
    #[error("merge-base default severity levels do not parse: {0}")]
    PriorLevels(serde_json::Error),
    #[error("merge-base scales: {0}")]
    PriorScales(ScaleFailure),
    #[error("standing budgets fail closed: meta/log is unreadable: {0}")]
    StandingLog(LogFailure),
    #[error("standing budgets fail closed: meta/log has no events on the remote")]
    NoLog,
    #[error("local gate fails closed: meta/log has no events on the remote")]
    GateNoLog,
    #[error("local gate fails closed: cannot fetch meta/log before the isolated run: {0}")]
    GateFetch(LogFailure),
    #[error("local gate fails closed: this run needs meta/log but the gate did not fetch it")]
    Unfetched,
    #[error("untracked work fails closed: {0}")]
    Untracked(Untracked),
    #[error("goldens: {0}")]
    Golden(#[from] GoldenFailure),
    #[error("goldens: cargo built no `{0}` binary")]
    Unbuilt(&'static str),
    #[error("git: {0}")]
    Git(Unread),
    #[error("{0}")]
    Repository(#[from] meta_git::GitFailure),
    #[error("{} is not inside a git repository", .0.display())]
    NoRepository(PathBuf),
    #[error("{0}")]
    BuildDirectory(#[from] Misplaced),
    #[error("{0}")]
    TrackedBuildOutput(TrackedBuildFiles),
    #[error("{failure}\nsupply chain fails closed: fix the finding above, or install {tool} if it is missing")]
    Supply { tool: &'static str, failure: Box<task::Failure> },
    #[error("{0}")]
    Adoption(Gaps),
    #[error("{skipped}: fails closed in CI: {failure}")]
    Unreached { skipped: String, failure: Box<Failure> },
}

#[cfg(test)]
mod tests {
    use task::{FailureKind, FailureSet, Printed};

    use super::Failure;

    const JOB: &str = "job";

    #[test]
    fn a_found_set_survives_wrapping_and_other_failures_name_nothing() {
        let found = FailureSet::default().recording(FailureKind::Infra { job: JOB.into() });
        let task = || task::Failure::Found { printed: Printed::Text(JOB.into()), found: found.clone() };
        let wrapped = Failure::Unreached { skipped: JOB.into(), failure: Box::new(Failure::from(task())) };
        assert!([Failure::from(task()), wrapped].iter().all(|failure| failure.found(&JOB.into()) == found));
        assert!(Failure::NoLog.found(&JOB.into()).is_empty());
    }
}
