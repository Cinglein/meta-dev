use std::io;

use thiserror::Error;

use crate::drift::Drift;
use crate::ratchet::Ratchet;
use crate::site::Site;

#[derive(Debug, Error)]
pub enum Failure {
    #[error("{site}: {error}")]
    Io { site: Site, error: io::Error },
    #[error("{site}: {error}")]
    Parse { site: Site, error: syn::Error },
    #[error("{site}: {error}")]
    Baseline { site: Site, error: Box<toml::de::Error> },
    #[error("baseline: {0}")]
    Serialize(toml::ser::Error),
    #[error("{0}")]
    Ratchet(Box<Ratchet>),
    #[error("baseline raised above the merge base:\n{}", .0.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"))]
    Raised(Vec<Drift>),
    #[error("no rust files to lint")]
    NoFiles,
    #[error("git: {0}")]
    Git(io::Error),
    #[error("no merge base with origin/HEAD; run `git fetch origin` and `git remote set-head origin --auto`")]
    NoDefaultBranch,
}
