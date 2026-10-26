use std::io;

use thiserror::Error;

use crate::task::ratchet::{Raised, Ratchet};

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
    #[error("no merge base with origin/HEAD; run `git fetch origin` and `git remote set-head origin --auto`")]
    NoDefaultBranch,
    #[error("step summary: {0}")]
    Summary(io::Error),
}
