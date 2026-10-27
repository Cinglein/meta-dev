use std::io;
use std::path::PathBuf;

use meta_log::LogFailure;
use meta_severity::Strands;
use thiserror::Error;

use crate::task::ci::{Busy, OutsideConfig, Unready};
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
    #[error("standing budgets fail closed: meta/log is unreadable: {0}")]
    StandingLog(LogFailure),
    #[error("standing budgets fail closed: meta/log has no events on the remote")]
    NoLog,
}
