use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Failure {
    #[error("{0}")]
    Usage(&'static str),
    #[error("no {0} in this directory or above")]
    Unconfigured(&'static str),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Config(#[from] toml::de::Error),
    #[error("{0}")]
    Lint(#[from] meta_lints::Failure),
    #[error("{0}")]
    Ci(#[from] meta_ci::Failure),
}
