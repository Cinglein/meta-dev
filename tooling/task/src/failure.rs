use std::io;

use thiserror::Error;

use crate::report::Report;
use crate::site::Site;

#[derive(Debug, Error)]
pub enum Failure {
    #[error("{0}")]
    Usage(String),
    #[error("unknown flag {flag}; the only flag is {known}")]
    UnknownFlag { flag: String, known: &'static str },
    #[error("no cargo workspace above the tooling")]
    NoWorkspace,
    #[error("{0} failed")]
    Command(String),
    #[error("cargo metadata is not the shape expected: {0}")]
    Metadata(#[from] serde_json::Error),
    #[error("{site}: {error}")]
    Io { site: Site, error: io::Error },
    #[error("{site}: {error}")]
    Parse { site: Site, error: syn::Error },
    #[error("{}", .0.iter().map(ToString::to_string).collect::<Vec<String>>().join("\n"))]
    Lint(Vec<Report>),
}
