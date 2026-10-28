use std::io;
use std::path::PathBuf;

use meta_log::LogFailure;
use meta_severity::{Demotions, ScaleFailure, Strands};
use thiserror::Error;

use crate::task::ci::{Busy, OutsideConfig, Unready, Untracked};
use crate::task::ratchet::{Raised, Ratchet};
use crate::task::revision::ForeignEmails;

#[derive(Debug, Error)]
pub enum Failure {
    #[error("{0}")]
    Task(#[from] task::Failure),
    #[error("{0}")]
    Ratchet(Ratchet),
    #[error("{0}")]
    Raised(Raised),
    #[error("baseline: {0}")]
    Baseline(#[from] toml::de::Error),
    #[error("baseline: {0}")]
    Written(#[from] toml::ser::Error),
    #[error("step summary: {0}")]
    Summary(io::Error),
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
    #[error("merge-base Cargo.lock does not parse: {0}")]
    PriorLock(toml::de::Error),
    #[error(
        "merge-base default severity levels fail closed: the meta-xtask commit pinned at the merge base is not in the cargo git databases; check out the merge base and run cargo fetch, or keep the pin unchanged"
    )]
    PinnedCommitMissing,
    #[error(
        "merge-base default severity levels fail closed: meta-xtask is not pinned by a git commit and its pin changed; pin it by git rev, or keep the pin unchanged"
    )]
    XtaskUnpinned,
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
}
